use std::collections::HashMap;
use std::io;
use std::time::Duration;

use mio::Interest;

use crate::net::SocketEvent;
use crate::net::transport::tcp_socket::TcpSocket;

pub struct SocketPoller {
    poll: mio::Poll,
    events: mio::Events,
}

impl SocketPoller {
    pub fn new() -> Self {
        let poll = match mio::Poll::new() {
            Ok(poll) => poll,
            Err(err) => panic!("could not create Poll, err: {:?}", err),
        };
        let events = mio::Events::with_capacity(256);
        Self { poll, events }
    }

    pub fn register_socket(&mut self, socket: &mut TcpSocket) -> io::Result<()> {
        self.poll.registry().register(
            socket,
            socket.token,
            Interest::READABLE | Interest::WRITABLE,
        )
    }

    pub fn deregister_socket(&mut self, socket: &mut TcpSocket) {
        self.poll.registry().deregister(socket);
    }

    pub fn tick(&mut self, timeout: Duration) -> HashMap<mio::Token, u8> {
        self.poll.poll(&mut self.events, Some(timeout));
        let mut triggers = HashMap::new();
        for event in &self.events {
            let mut trigger = 0;
            if event.is_readable() {
                trigger |= SocketEvent::Readable as u8;
            }
            if event.is_writable() {
                trigger |= SocketEvent::Writable as u8;
            }

            *triggers.entry(event.token()).or_insert(trigger) |= trigger;
        }
        triggers
    }
}
