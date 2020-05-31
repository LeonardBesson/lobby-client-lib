use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::time::{Duration, Instant};
use std::{io, mem};

use bytes::{Bytes, BytesMut};
use mio::net::TcpStream;

use crate::net::packet::{packet_to_message, Packet};
use crate::net::packets::ClientInitRequest;
use crate::net::socket_poller::SocketPoller;
use crate::net::transport::tcp_socket::TcpSocket;
use crate::net::SocketEvent;
use crate::utils::buffer_processor::LogBufferProcessor;
use crate::utils::byte_buffer::ByteBuffer;

pub struct SocketManager {
    poller: SocketPoller,
    sockets: Vec<TcpSocket>,
    free_tokens: VecDeque<mio::Token>,
    tokens: HashMap<SocketAddr, mio::Token>,
    recv_buffer: Vec<u8>,
    flushables: HashSet<mio::Token>,
}

impl SocketManager {
    pub fn new() -> Self {
        Self {
            poller: SocketPoller::new(),
            sockets: Vec::new(),
            free_tokens: VecDeque::new(),
            tokens: HashMap::new(),
            recv_buffer: vec![0; 4096],
            flushables: HashSet::new(),
        }
    }

    fn new_tcp_socket(&mut self, addr: SocketAddr) -> io::Result<&mut TcpSocket> {
        let stream = TcpStream::connect(addr)?;
        let token = if let Some(token) = self.free_tokens.pop_front() {
            token
        } else {
            mio::Token(self.sockets.len())
        };
        let mut socket = TcpSocket::new(stream, token);
        socket.add_buffer_processor(Box::new(LogBufferProcessor));
        self.poller.register_socket(&mut socket)?;
        self.sockets.insert(token.0, socket);
        self.tokens.insert(addr, token);
        Ok(&mut self.sockets[token.0])
    }

    fn remove_tcp_socket(&mut self, token: mio::Token) {
        if let Some(sock) = self.sockets.get_mut(token.0) {
            sock.stream.shutdown(Shutdown::Both);
            self.poller.deregister_socket(sock);
            self.free_tokens.push_back(token);

            match sock.stream.peer_addr() {
                Ok(addr) => {
                    self.tokens.remove(&addr);
                }
                Err(err) => {
                    println!("Could not get peer_addr on socket to remove, {:?}", err);
                    if let Some(addr) = self
                        .tokens
                        .iter()
                        .find(|(_, &t)| t == token)
                        .map(|(&addr, _)| addr)
                    {
                        self.tokens.remove(&addr);
                    }
                }
            }
            // TODO: Make sure this works correctly: we shouldn't need to remove the socket
            // just free the token, and the next one created will take its place
        }
    }

    pub fn send(&mut self, peer: SocketAddr, packet: Packet) {
        let token = if let Some(token) = self.tokens.get(&peer) {
            self.sockets[token.0].send_packet(packet);
            *token
        } else {
            println!("No socket connected to {} yet, creating.", peer);
            let mut socket = match self.new_tcp_socket(peer) {
                Ok(s) => s,
                Err(err) => {
                    println!("Could not create new tcp socket: {:?}", err);
                    return;
                }
            };
            socket.send_packet(packet);
            socket.token
        };
        self.flushables.insert(token);
    }

    fn flush(&mut self, token: mio::Token) {
        if let Some(sock) = self.sockets.get_mut(token.0) {
            // Out
            while let Some(buffer) = sock.tcp_encoder.next_buffer() {
                sock.unprocessed_out.push_back(buffer);
            }
            sock.process_out();

            // In
            sock.process_in();
            while let Some(buffer) = sock.processed_in.pop_front() {
                sock.tcp_decoder.push_buffer(buffer);
            }

            if !sock.processed_out.is_empty() {
                self.try_to_send(token);
            }
        }
    }

    fn try_to_read(&mut self, token: mio::Token) {
        let sock = self.sockets.get_mut(token.0).expect(
            "Received trigger for non-existent token.\
             It probably wasn't deregistered from poller before being removed.",
        );
        loop {
            match sock.stream.read(&mut self.recv_buffer) {
                Ok(recv_len) => {
                    println!("Read {} bytes", recv_len);
                    if recv_len > 0 {
                        println!("Read buffer: {:?}", &self.recv_buffer[..recv_len]);
                        sock.unprocessed_in
                            .push_back(Bytes::copy_from_slice(&self.recv_buffer[..recv_len]).into())
                    } else {
                        println!("Read 0, removing token {}", token.0);
                        self.remove_tcp_socket(token);
                        break;
                    }
                }
                Err(e) => {
                    println!("Read error: {:?}", e);
                    match e.kind() {
                        io::ErrorKind::ConnectionReset => {
                            self.remove_tcp_socket(token);
                        }
                        io::ErrorKind::WouldBlock => {}
                        _ => {
                            // What should we do with errors here?
                        }
                    }
                    break;
                }
            }
        }
        self.flushables.insert(token);
    }

    fn try_to_send(&mut self, token: mio::Token) {
        let sock = self.sockets.get_mut(token.0).expect(
            "Received trigger for non-existent token.\
             It probably wasn't deregistered from poller before being removed.",
        );
        if !sock.initialized {
            sock.stream
                .set_nodelay(true)
                .expect("Could not set nodelay on socket");
            sock.initialized = true;
            println!("Set nodelay for socket {}", token.0);
        }

        sock.process_out();
        println!("Writable, processed_out len: {}", sock.processed_out.len());
        while let Some(buffer) = sock.processed_out.front_mut() {
            match sock.stream.write(&buffer[..]) {
                Ok(n) if n < buffer.len() => {
                    println!("Written {} bytes, truncating buffer", n);
                    buffer.skip(n);
                }
                Ok(n) => {
                    println!("Written {} bytes", n);
                    sock.processed_out.pop_front();
                }
                Err(e) => {
                    println!("Write error: {:?}", e);
                    match e.kind() {
                        io::ErrorKind::ConnectionReset
                        | io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::ConnectionAborted
                        | io::ErrorKind::BrokenPipe => {
                            println!("Broken connection to {:?}", sock.stream.peer_addr());
                            self.remove_tcp_socket(token);
                        }
                        io::ErrorKind::WouldBlock => {}
                        _ => {
                            // handle other errors
                        }
                    }
                    break;
                }
            }
        }
    }

    pub fn tick(&mut self, timeout: Duration) {
        let triggers = self.poller.tick(timeout);
        for (&token, &trigger) in triggers.iter() {
            if (trigger & SocketEvent::Readable as u8) != 0 {
                self.try_to_read(token);
            }
            if (trigger & SocketEvent::Writable as u8) != 0 {
                self.try_to_send(token);
            }
            if (trigger & SocketEvent::Closed as u8) != 0 {
                self.remove_tcp_socket(token);
            }
        }

        let flushables = mem::replace(&mut self.flushables, HashSet::new());
        for token in flushables {
            self.flush(token);
        }
        assert!(self.flushables.is_empty());
    }
}
