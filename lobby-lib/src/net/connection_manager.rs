use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Error, Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::time::{Duration, Instant};
use std::{io, mem};

use bytes::{Bytes, BytesMut};
use mio::net::TcpStream;

use crate::net::connection::Connection;
use crate::net::packet::{packet_to_message, Packet};
use crate::net::socket_poller::SocketPoller;
use crate::net::transport::tcp_socket::TcpSocket;
use crate::net::SocketEvent;
use crate::utils::buffer_processor::LogBufferProcessor;
use crate::utils::byte_buffer::ByteBuffer;
use crate::LobbyEvent;

pub struct ConnectionManager {
    poller: SocketPoller,
    connections: Vec<Connection>,
    free_tokens: VecDeque<mio::Token>,
    tokens: HashMap<SocketAddr, mio::Token>,
    recv_buffer: Vec<u8>,
    flushables: HashSet<mio::Token>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            poller: SocketPoller::new(),
            connections: Vec::new(),
            free_tokens: VecDeque::new(),
            tokens: HashMap::new(),
            recv_buffer: vec![0; 4096],
            flushables: HashSet::new(),
        }
    }

    /// Init a connection to the given address
    pub fn connect(&mut self, addr: SocketAddr) {
        if let Err(err) = self.new_connection(addr) {
            println!("Could not create new connection: {:?}", err);
        }
    }

    fn new_connection(&mut self, addr: SocketAddr) -> io::Result<&mut Connection> {
        let token = self
            .free_tokens
            .pop_front()
            .unwrap_or_else(|| mio::Token(self.connections.len()));
        let mut conn = Connection::open(addr, token)?;
        conn.add_buffer_processor(Box::new(LogBufferProcessor));
        self.poller.register_connection(&mut conn)?;
        self.connections.insert(token.0, conn);
        self.tokens.insert(addr, token);
        Ok(&mut self.connections[token.0])
    }

    fn close_connection(&mut self, token: mio::Token) {
        if let Some(mut conn) = self.connections.get_mut(token.0) {
            conn.close();
            self.poller.deregister_connection(&mut conn);
            self.free_tokens.push_back(token);

            self.tokens.remove(&conn.peer_info.addr);
            // TODO: Make sure this works correctly: we shouldn't need to remove the socket
            // just free the token, and the next one created will take its place
        }
    }

    pub fn send(&mut self, peer: SocketAddr, packet: Packet) {
        let token = if let Some(token) = self.tokens.get(&peer) {
            self.connections[token.0].send(packet);
            *token
        } else {
            println!("No connection to {} yet, initiating.", peer);
            let mut conn = match self.new_connection(peer) {
                Ok(s) => s,
                Err(err) => {
                    println!("Could not create new connection: {:?}", err);
                    return;
                }
            };
            conn.send(packet);
            conn.token
        };
        self.flushables.insert(token);
    }

    fn should_close(err_kind: io::ErrorKind) -> bool {
        match err_kind {
            io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe => true,
            _ => false,
        }
    }

    fn readable(&mut self, token: mio::Token) {
        let conn = self.connections.get_mut(token.0).expect(
            "Received trigger for non-existent token.\
             It probably wasn't deregistered from poller before being removed.",
        );
        if let Err(err) = conn.read(&mut self.recv_buffer) {
            match err.kind() {
                io::ErrorKind::WouldBlock => {}
                kind if Self::should_close(kind) => {
                    println!("Closing connection due to {:?}", err);
                    self.close_connection(token);
                    return;
                }
                _ => {
                    println!("Unhandled read error {:?}", err);
                }
            }
        }
        self.flushables.insert(token);
    }

    fn writable(&mut self, token: mio::Token) {
        if let Some(conn) = self.connections.get_mut(token.0) {
            if let Err(err) = conn.write() {
                match err.kind() {
                    io::ErrorKind::WouldBlock => {}
                    kind if Self::should_close(kind) => {
                        println!("Closing connection due to {:?}", err);
                        self.close_connection(token);
                        return;
                    }
                    _ => {
                        println!("Unhandled write error {:?}", err);
                    }
                }
            }
        }
    }

    pub fn tick(&mut self, incoming_events: &mut VecDeque<LobbyEvent>, timeout: Duration) {
        let triggers = self.poller.tick(timeout);
        for (&token, &trigger) in triggers.iter() {
            if (trigger & SocketEvent::Readable as u8) != 0 {
                self.readable(token);
            }
            if (trigger & SocketEvent::Writable as u8) != 0 {
                self.writable(token);
            }
            if (trigger & SocketEvent::Closed as u8) != 0 {
                self.close_connection(token);
            }
        }

        if !self.flushables.is_empty() {
            let flushables = mem::replace(&mut self.flushables, HashSet::new());
            for token in flushables {
                if let Some(conn) = self.connections.get_mut(token.0) {
                    conn.flush();
                    if conn.has_events() {
                        incoming_events.extend(conn.drain_events());
                    }
                }
            }
        }
        assert!(self.flushables.is_empty());
    }
}
