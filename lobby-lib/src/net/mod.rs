use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::ops::Range;
use std::time::{Duration, Instant};
use std::{io, mem};

use bytes::Bytes;
use mio::net::TcpStream;
use mio::{Interest, Registry, Token};
use serde::{Deserialize, Serialize};

use crate::net::connection_manager::ConnectionManager;
use crate::net::packet::{message_to_packet, Packet, PacketInfo};
use crate::net::packet_encoder::PacketEncoder;
use crate::net::packets::{PacketInit, PacketType};
use crate::net::transport::tcp_socket::TcpSocket;
use crate::utils::byte_buffer::ByteBuffer;
use crate::LobbyEvent;

pub mod connection;
pub mod connection_manager;
pub mod packet;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod packets;
pub mod socket_poller;
pub mod transport;

pub type Result<T> = ::std::result::Result<T, Error>;

pub type Error = Box<ErrorKind>;

#[derive(Debug)]
pub enum ErrorKind {
    Serialize(String),
    Deserialize(String),
    InvalidPacketType(PacketType),
}

pub trait Message<'de>: Serialize + Deserialize<'de> {
    fn packet_type(&self) -> PacketType;
    fn packet_info(&self) -> PacketInfo;
    fn serialize_data(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self)
    }
    fn deserialize(buffer: &'de [u8]) -> bincode::Result<Self> {
        bincode::deserialize(&buffer[..])
    }
}

#[repr(u8)]
enum SocketEvent {
    Readable = 1 << 0,
    Writable = 1 << 1,
    Closed = 1 << 2,
}

pub struct Net {
    pub connection_manager: ConnectionManager,
    incoming_events: VecDeque<LobbyEvent>,
}

impl Net {
    pub fn new() -> Self {
        Self {
            connection_manager: ConnectionManager::new(),
            incoming_events: VecDeque::new(),
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) {
        self.connection_manager.connect(addr);
    }

    pub fn tick(&mut self, timeout: Duration) {
        self.connection_manager
            .tick(&mut self.incoming_events, timeout);
    }

    pub fn poll_events(&mut self, events: &mut Vec<LobbyEvent>) {
        events.clear();
        loop {
            if self.incoming_events.is_empty() || events.len() >= events.capacity() {
                break;
            }
            if let Some(event) = self.incoming_events.pop_front() {
                events.push(event);
            }
        }
    }

    pub fn send_message<'de, T: Message<'de>>(&mut self, peer: SocketAddr, message: &T) {
        let packet_type = message.packet_type();
        match message_to_packet(message) {
            Ok(packet) => {
                self.send_packet(peer, packet);
            }
            Err(err) => {
                println!("Could not convert message {:?} to packet", packet_type);
                return;
            }
        }
    }

    pub fn send_packet(&mut self, peer: SocketAddr, packet: Packet) {
        self.connection_manager.send(peer, packet);
    }
}
