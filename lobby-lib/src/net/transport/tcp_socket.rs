use std::collections::VecDeque;
use std::io;

use mio::net::TcpStream;
use mio::{Interest, Registry, Token};

use crate::net::packet::Packet;
use crate::net::packet_decoder::PacketDecoder;
use crate::net::packet_encoder::PacketEncoder;
use crate::utils::buffer_processor::{BufferProcessor, Direction};
use crate::utils::byte_buffer::ByteBuffer;
use std::net::Shutdown;

pub struct TcpSocket {
    pub stream: TcpStream,
    connected: bool,

    buffer_processors: Vec<Box<dyn BufferProcessor>>,
    pub unprocessed_in: VecDeque<ByteBuffer>,
    pub unprocessed_out: VecDeque<ByteBuffer>,
    pub processed_in: VecDeque<ByteBuffer>,
    pub processed_out: VecDeque<ByteBuffer>,
}

impl TcpSocket {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            connected: false,
            buffer_processors: Vec::new(),
            unprocessed_in: VecDeque::new(),
            unprocessed_out: VecDeque::new(),
            processed_in: VecDeque::new(),
            processed_out: VecDeque::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn connected(&mut self) {
        self.stream
            .set_nodelay(true)
            .expect("Could not set nodelay on socket");
        self.connected = true;
    }

    pub fn close(&mut self) {
        if self.connected {
            self.stream.shutdown(Shutdown::Both);
            self.connected = false;
        }
    }

    pub fn add_buffer_processor(&mut self, buffer_processor: Box<dyn BufferProcessor>) {
        self.buffer_processors.push(buffer_processor);
    }

    pub fn process_in(&mut self) {
        while let Some(mut buffer) = self.unprocessed_in.pop_front() {
            for processor in self.buffer_processors.iter_mut().rev() {
                processor.process_buffer(&mut buffer, Direction::In);
            }
            self.processed_in.push_back(buffer);
        }
    }
    pub fn process_out(&mut self) {
        while let Some(mut buffer) = self.unprocessed_out.pop_front() {
            for processor in self.buffer_processors.iter_mut() {
                processor.process_buffer(&mut buffer, Direction::Out);
            }
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
