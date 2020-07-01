use crate::net::packet::{message_to_packet, packet_to_message, Packet};
use crate::net::packet_decoder::PacketDecoder;
use crate::net::packet_encoder::PacketEncoder;
use crate::net::packets::*;
use crate::net::transport::tcp_socket::TcpSocket;
use crate::utils::buffer_processor::BufferProcessor;
use crate::utils::time;
use crate::{net, ErrorCode, LobbyEvent};
use bytes::Bytes;
use log::{debug, error};
use mio::net::TcpStream;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{io, mem};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ConnState {
    Initializing,
    Authenticating,
    Running,
    Closed,
}

pub struct PeerInfo {
    pub addr: SocketAddr,
}

impl PeerInfo {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

pub struct Connection {
    pub token: mio::Token,
    pub peer_info: PeerInfo,
    pub state: ConnState,
    pub closed_time: Option<Instant>,

    pub socket: TcpSocket,
    pub tcp_encoder: PacketEncoder,
    pub tcp_decoder: PacketDecoder,

    events: Vec<LobbyEvent>,
}

impl Connection {
    /// Create the connection and issue non blocking connect
    pub fn open(addr: SocketAddr, token: mio::Token) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let mut socket = TcpSocket::new(stream);
        let mut conn = Self {
            token,
            peer_info: PeerInfo::new(addr),
            state: ConnState::Initializing,
            closed_time: None,
            socket,
            tcp_encoder: PacketEncoder::new(8 * 1024),
            tcp_decoder: PacketDecoder::new(),
            events: Vec::new(),
        };
        // Init handshake
        conn.send(
            message_to_packet(&PacketInit {
                protocol_version: crate::PROTOCOL_VERSION,
                app_version: crate::APP_VERSION,
            })
            .unwrap(),
        );
        Ok(conn)
    }

    /// Add a buffer processor to be executed when sending and receiving buffers.
    /// Buffer processors are ran in added order for outbound, and backwards for inbound.
    /// So if you first add a processor to do encryption and then another to do compression,
    /// Outgoing packets will be encrypted, then compressed, whereas incoming packets will
    /// be decompressed, and then decrypted.
    pub fn add_buffer_processor(&mut self, buffer_processor: Box<dyn BufferProcessor>) {
        self.socket.add_buffer_processor(buffer_processor);
    }

    pub fn send(&mut self, packet: Packet) {
        self.tcp_encoder.add_packet(packet);
    }

    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    pub fn drain_events(&mut self) -> Vec<LobbyEvent> {
        mem::replace(&mut self.events, Vec::new())
    }

    pub fn flush(&mut self) {
        // Out
        while let Some(buffer) = self.tcp_encoder.next_buffer() {
            self.socket.unprocessed_out.push_back(buffer);
        }
        self.socket.process_out();

        if !self.socket.processed_out.is_empty() {
            self.write();
        }

        // In
        self.socket.process_in();
        while let Some(buffer) = self.socket.processed_in.pop_front() {
            self.tcp_decoder.push_buffer(buffer);
        }
        while let Some(packet) = self.tcp_decoder.next_packet() {
            self.incoming_packet(packet);
        }
    }

    pub fn close(&mut self) {
        self.socket.close();
        self.state = ConnState::Closed;
        self.closed_time = Some(Instant::now());
    }

    /// Read as much as possible from the connection's socket.
    /// Buffers are read into the given buffer, and pushed to be processed.
    pub fn read(&mut self, read_buffer: &mut [u8]) -> io::Result<()> {
        loop {
            let res = self.socket.stream.read(read_buffer);
            if let Ok(n) = res {
                debug!("Read {} bytes", n);
                if n > 0 {
                    debug!("Read buffer: {:?}", &read_buffer[..n]);
                    self.socket
                        .unprocessed_in
                        .push_back(Bytes::copy_from_slice(&read_buffer[..n]).into())
                } else {
                    return Err(io::ErrorKind::ConnectionAborted.into());
                }
            } else {
                return res.map(|_| ());
            }
        }
    }

    /// Process out buffers and write as much as possible to the connection's socket.
    pub fn write(&mut self) -> io::Result<()> {
        if !self.socket.is_connected() {
            // Connected (i.e we received the first write event after connect)
            self.socket.connected();
            debug!("Set nodelay for connection {}", self.token.0);
        }

        self.socket.process_out();
        debug!(
            "Writable, processed_out len: {}",
            self.socket.processed_out.len()
        );
        while let Some(buffer) = self.socket.processed_out.front_mut() {
            let res = self.socket.stream.write(&buffer[..]);
            match res {
                Ok(n) if n < buffer.len() => {
                    debug!("Written {} bytes, truncating buffer", n);
                    buffer.skip(n);
                }
                Ok(n) => {
                    debug!("Written {} bytes", n);
                    self.socket.processed_out.pop_front();
                }
                _ => return res.map(|_| ()),
            }
        }
        Ok(())
    }

    fn incoming_packet(&mut self, packet: Packet) {
        debug!("Handling packet {:?}", packet.packet_type);
        match packet.packet_type {
            PacketType::PacketInit => {
                let msg = packet_to_message::<PacketInit>(&packet).unwrap();
                if msg.app_version != crate::APP_VERSION {
                    self.disconnect("Invalid app version");
                    return;
                }
                if msg.protocol_version != crate::PROTOCOL_VERSION {
                    self.disconnect("Invalid protocol version");
                    return;
                }
                self.state = ConnState::Authenticating;
                self.events.push(LobbyEvent::ConnectionEstablished);
            }
            PacketType::FatalError => {
                let msg = packet_to_message::<FatalError>(&packet).unwrap();
                self.disconnect(&msg.message);
                return;
            }
            PacketType::PacketPing => {
                let msg = packet_to_message::<PacketPing>(&packet).unwrap();
                self.send(
                    message_to_packet(&PacketPong {
                        id: msg.id,
                        peer_time: time::unix_millis(),
                    })
                    .unwrap(),
                );
                self.flush();
            }
            PacketType::AuthenticationResponse => {
                let msg = packet_to_message::<AuthenticationResponse>(&packet).unwrap();
                match msg {
                    AuthenticationResponse {
                        error_code: Some(err),
                        session_token: None,
                        user_profile: None,
                    } => self.events.push(LobbyEvent::AuthFailure {
                        error_code: ErrorCode::from_str(&err).expect("unknown error code"),
                    }),
                    AuthenticationResponse {
                        error_code: None,
                        session_token: Some(session_token),
                        user_profile: Some(user_profile),
                    } => self.events.push(LobbyEvent::AuthSuccess {
                        session_token,
                        user_profile,
                    }),
                    _ => self.disconnect("Protocol error"),
                }
            }
            PacketType::FetchPendingFriendRequestsResponse => {
                let msg = packet_to_message::<FetchPendingFriendRequestsResponse>(&packet).unwrap();
                self.events.push(LobbyEvent::FriendRequestsUpdated {
                    as_inviter: msg.pending_as_inviter,
                    as_invitee: msg.pending_as_invitee,
                });
            }
            PacketType::FetchFriendListResponse => {
                let msg = packet_to_message::<FetchFriendListResponse>(&packet).unwrap();
                self.events.push(LobbyEvent::FriendListUpdated {
                    friend_list: msg.friend_list,
                });
            }
            PacketType::NewPrivateMessage => {
                let msg = packet_to_message::<NewPrivateMessage>(&packet).unwrap();
                self.events.push(LobbyEvent::NewPrivateMessage {
                    profile: msg.profile,
                    content: msg.content,
                    is_self: msg.is_self,
                });
            }
            PacketType::SystemNotification => {
                let msg = packet_to_message::<SystemNotification>(&packet).unwrap();
                self.events.push(LobbyEvent::SystemNotification {
                    content: msg.content,
                });
            }
            PacketType::LobbyInvite => {
                let msg = packet_to_message::<LobbyInvite>(&packet).unwrap();
                self.events.push(LobbyEvent::LobbyInvite {
                    id: msg.id,
                    inviter: msg.inviter,
                });
            }
            _ => {
                error!("Received unhandled packet type: {:?}", packet.packet_type);
            }
        }
    }

    fn disconnect(&mut self, error_message: &str) {
        if self.socket.is_connected() {
            self.send(
                message_to_packet(&FatalError {
                    message: error_message.to_owned(),
                })
                .unwrap(),
            );
            self.flush();
            self.close();
            self.events.push(LobbyEvent::Disconnected {
                message: error_message.to_owned(),
            });
        }
    }
}
