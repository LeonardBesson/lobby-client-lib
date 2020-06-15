#[macro_use]
extern crate lazy_static;

use std::net::SocketAddr;
use std::time::Duration;

use crate::net::connection::{ConnState, Connection};
use crate::net::packets::*;
use crate::net::{Message, Net};

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
    net: Net,
    events: Vec<LobbyEvent>,
}

impl LobbyClient {
    pub fn new(url: &str) -> Result<Self> {
        let addr = url
            .parse()
            .map_err(|_| ErrorKind::InvalidArg(format!("Invalid url {}", url)))?;
        Ok(Self {
            addr,
            net: Net::new(),
            events: Vec::new(),
        })
    }

    pub fn connect(&mut self) {
        self.net.connect(self.addr);
    }

    pub fn disconnect(&mut self, free: bool) {
        self.net.connection_manager.disconnect(self.addr, free);
    }

    pub fn tick(&mut self, timeout: Duration) {
        self.net.tick(timeout);
    }

    pub fn poll_events(&mut self, events: &mut Vec<LobbyEvent>) {
        self.net.poll_events(events);
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
        self.net.send_message(self.addr, &message);
    }

    fn initialized(&mut self) -> bool {
        self.connection_mut().state >= ConnState::Authenticating
    }

    fn closed(&mut self) -> bool {
        self.connection_mut().state == ConnState::Closed
    }

    fn connection_mut(&mut self) -> &mut Connection {
        self.net
            .connection_manager
            .connect_mut(self.addr)
            .expect("connect() never called")
    }
}
