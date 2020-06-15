#[macro_use]
extern crate lazy_static;

use std::net::SocketAddr;
use std::time::Duration;

use crate::net::connection::{ConnState, Connection};
use crate::net::connection_manager::ConnectionManager;
use crate::net::packet::{message_to_packet, Packet};
use crate::net::packets::*;
use crate::net::Message;
use std::collections::VecDeque;

pub const PROTOCOL_VERSION: u16 = 1;
pub const APP_VERSION: u16 = 1;

pub mod net;
pub mod utils;

#[derive(Debug, Clone)]
pub enum LobbyEvent {
    Connected,
    Disconnected { message: String },
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub type Error = Box<ErrorKind>;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidArg(String),
}

pub struct LobbyClient {
    addr: SocketAddr,
    connection_manager: ConnectionManager,
    incoming_events: VecDeque<LobbyEvent>,
}

impl LobbyClient {
    pub fn new(url: &str) -> Result<Self> {
        let addr = url
            .parse()
            .map_err(|_| ErrorKind::InvalidArg(format!("Invalid url {}", url)))?;
        Ok(Self {
            addr,
            connection_manager: ConnectionManager::new(),
            incoming_events: VecDeque::new(),
        })
    }

    pub fn connect(&mut self) {
        self.connection_manager.connect(self.addr);
    }

    pub fn disconnect(&mut self, free: bool) {
        self.connection_manager.disconnect(self.addr, free);
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

    pub fn authenticate(&mut self, username: String, password: String) {
        if !self.initialized() {
            println!("authenticate() called before initialized");
            return;
        }
        if self.closed() {
            println!("authenticate() called when closed");
            return;
        }
        self.send_to_lobby(AuthenticationRequest { username, password });
    }

    fn send_to_lobby<'de, T: Message<'de>>(&mut self, message: T) {
        self.send_message(self.addr, &message);
    }

    fn initialized(&mut self) -> bool {
        self.connection_mut().state >= ConnState::Authenticating
    }

    fn closed(&mut self) -> bool {
        self.connection_mut().state == ConnState::Closed
    }

    fn connection_mut(&mut self) -> &mut Connection {
        self.connection_manager
            .connect_mut(self.addr)
            .expect("connect() never called")
    }

    fn send_message<'de, T: Message<'de>>(&mut self, peer: SocketAddr, message: &T) {
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

    fn send_packet(&mut self, peer: SocketAddr, packet: Packet) {
        self.connection_manager.send(peer, packet);
    }
}
