use crate::net::connection_manager::ConnectionManager;
use crate::net::packet::{message_to_packet, Packet, PacketInfo};
use crate::net::packet_encoder::PacketEncoder;
use crate::net::packets::{PacketInit, PacketType};
use crate::net::transport::tcp_socket::TcpSocket;
use crate::utils::byte_buffer::ByteBuffer;
use crate::LobbyEvent;
use bincode::config::Options;
use bytes::Bytes;
use mio::net::TcpStream;
use mio::{Interest, Registry, Token};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::ops::Range;
use std::time::{Duration, Instant};
use std::{io, mem};

pub mod connection;
pub mod connection_manager;
pub mod packet;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod packets;
pub mod socket_poller;
pub mod structs;
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
        bincode::config::DefaultOptions::new().serialize(&self)
    }
    fn deserialize(buffer: &'de [u8]) -> bincode::Result<Self> {
        bincode::config::DefaultOptions::new().deserialize(&buffer[..])
    }
}

#[repr(u8)]
enum SocketEvent {
    Readable = 1 << 0,
    Writable = 1 << 1,
    Closed = 1 << 2,
}
