use std::collections::VecDeque;
use std::io;

use mio::{Interest, Registry, Token};
use mio::net::TcpStream;

use crate::net::packet::Packet;
use crate::net::packet_decoder::PacketDecoder;
use crate::net::packet_encoder::PacketEncoder;
use crate::utils::byte_buffer::ByteBuffer;

pub struct TcpSocket {
    pub stream: TcpStream,
    pub token: mio::Token,
    pub initialized: bool,
    pub tcp_encoder: PacketEncoder,
    pub tcp_decoder: PacketDecoder,

    pub unprocessed_in: VecDeque<ByteBuffer>,
    pub unprocessed_out: VecDeque<ByteBuffer>,
    pub processed_in: VecDeque<ByteBuffer>,
    pub processed_out: VecDeque<ByteBuffer>,
}

impl TcpSocket {
    pub fn new(stream: TcpStream, token: mio::Token) -> Self {
        Self {
            stream,
            token,
            initialized: false,
            tcp_encoder: PacketEncoder::new(8 * 1024),
            tcp_decoder: PacketDecoder::new(),
            unprocessed_in: VecDeque::new(),
            unprocessed_out: VecDeque::new(),
            processed_in: VecDeque::new(),
            processed_out: VecDeque::new(),
        }
    }

    pub fn token(&self) -> mio::Token {
        self.token
    }

    pub fn send_packet(&mut self, packet: Packet) {
        println!("Sending packet of length {}", packet.data_size());
        self.tcp_encoder.add_packet(packet);
    }

    pub fn next_packet(&mut self) -> Option<Packet> {
        self.process_in();
        self.tcp_decoder.next_packet()
    }

    pub fn process_in(&mut self) {
        // TODO: processors
        while let Some(buffer) = self.unprocessed_in.pop_front() {
            self.processed_in.push_back(buffer);
        }
    }
    pub fn process_out(&mut self) {
        // TODO: processors
        while let Some(buffer) = self.unprocessed_out.pop_front() {
            self.processed_out.push_back(buffer);
        }
    }
}

impl mio::event::Source for TcpSocket {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.stream.register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.stream.reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        self.stream.deregister(registry)
    }
}
