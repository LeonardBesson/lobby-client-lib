#[macro_use]
extern crate lazy_static;
use crate::net::connection::{ConnState, Connection};
use crate::net::connection_manager::ConnectionManager;
use crate::net::packet::{message_to_packet, Packet};
use crate::net::packets::*;
use crate::net::structs::{Friend, FriendRequest, FriendRequestActionChoice, UserProfile};
use crate::net::Message;
use log::{debug, error};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub const PROTOCOL_VERSION: u16 = 1;
pub const APP_VERSION: u16 = 1;

pub mod net;
pub mod utils;

#[derive(Debug, Copy, Clone)]
pub enum ErrorCode {
    InvalidCredentials,
}

impl FromStr for ErrorCode {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "invalid_credentials" => Ok(ErrorCode::InvalidCredentials),
            _ => Err(ErrorKind::InvalidArg(format!("Unknown error code: {}", input)).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LobbyEvent {
    ConnectionEstablished,
    Disconnected {
        message: String,
    },
    AuthSuccess {
        session_token: String,
        user_profile: UserProfile,
    },
    AuthFailure {
        error_code: ErrorCode,
    },
    FriendRequestsUpdated {
        as_invitee: Vec<FriendRequest>,
        as_inviter: Vec<FriendRequest>,
    },
    FriendListUpdated {
        friend_list: Vec<Friend>,
    },
    // TODO: error events
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub type Error = Box<ErrorKind>;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidArg(String),
}

pub struct LobbyClient {
    addr: SocketAddr,
    reconnect_interval: Option<Duration>,
    last_reconnect_attempt: Option<Instant>,
    connection_manager: ConnectionManager,
    incoming_events: VecDeque<LobbyEvent>,
}

pub struct LobbyClientBuilder<'a> {
    url: &'a str,
    reconnect_interval: Option<Duration>,
}

impl<'a> LobbyClientBuilder<'a> {
    pub fn new(url: &'a str) -> Self {
        Self {
            url,
            reconnect_interval: None,
        }
    }

    pub fn with_reconnect_interval(mut self, interval: Duration) -> Self {
        self.reconnect_interval = Some(interval);
        self
    }

    pub fn build(&self) -> Result<LobbyClient> {
        let addr = self
            .url
            .parse()
            .map_err(|_| ErrorKind::InvalidArg(format!("Invalid url {}", self.url)))?;
        Ok(LobbyClient {
            addr,
            reconnect_interval: self.reconnect_interval,
            last_reconnect_attempt: None,
            connection_manager: ConnectionManager::new(),
            incoming_events: VecDeque::new(),
        })
    }
}

impl LobbyClient {
    pub fn connect(&mut self) {
        self.connection_manager.connect(self.addr);
    }

    pub fn disconnect(&mut self, free: bool) {
        self.connection_manager.disconnect(self.addr, free);
    }

    pub fn tick(&mut self, timeout: Duration) {
        self.try_to_reconnect();
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
                self.handle_event(&event);
                events.push(event);
            }
        }
    }

    pub fn authenticate(&mut self, email: String, password: String) {
        if !self.initialized() {
            error!("authenticate() called before initialized");
            return;
        }
        if self.closed() {
            error!("authenticate() called when closed");
            return;
        }
        self.send_to_lobby(AuthenticationRequest { email, password });
    }

    pub fn add_friend(&mut self, user_tag: String) {
        self.send_to_lobby(AddFriendRequest { user_tag });
    }

    pub fn refresh_friend_requests(&mut self) {
        self.send_to_lobby(FetchPendingFriendRequests {});
    }

    pub fn refresh_friend_list(&mut self) {
        self.send_to_lobby(FetchFriendList {});
    }

    pub fn friend_request_action(&mut self, request_id: String, action: FriendRequestActionChoice) {
        self.send_to_lobby(FriendRequestAction { request_id, action });
    }

    pub fn remove_friend(&mut self, user_tag: String) {
        self.send_to_lobby(RemoveFriend { user_tag });
    }

    fn handle_event(&mut self, event: &LobbyEvent) {
        match event {
            LobbyEvent::ConnectionEstablished => {
                self.last_reconnect_attempt = None;
            }
            _ => {}
        }
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
                error!("Could not convert message {:?} to packet", packet_type);
                return;
            }
        }
    }

    fn send_packet(&mut self, peer: SocketAddr, packet: Packet) {
        self.connection_manager.send(peer, packet);
    }

    fn try_to_reconnect(&mut self) {
        if !self.closed() {
            return;
        }

        if let Some(interval) = self.reconnect_interval {
            let closed_at = self.connection_mut().closed_time;
            match (self.last_reconnect_attempt, closed_at) {
                (Some(attempt), _) if Instant::now() > attempt + interval => {
                    debug!("Reconnecting");
                    self.last_reconnect_attempt = Some(Instant::now());
                    self.connect();
                }
                (None, Some(closed_at)) if Instant::now() > closed_at + interval => {
                    debug!("Reconnecting");
                    self.last_reconnect_attempt = Some(Instant::now());
                    self.connect();
                }
                _ => {}
            }
        }
    }
}
