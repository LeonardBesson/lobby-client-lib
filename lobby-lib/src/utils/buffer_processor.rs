use crate::utils::byte_buffer::ByteBuffer;
use log::info;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    In,
    Out,
}

pub trait BufferProcessor {
    fn process_buffer(&mut self, buffer: &mut ByteBuffer, direction: Direction);
}

pub struct LogBufferProcessor;

impl BufferProcessor for LogBufferProcessor {
    fn process_buffer(&mut self, buffer: &mut ByteBuffer, direction: Direction) {
        match direction {
            Direction::In => {
                info!("Processing received buffer {:?}", &buffer[..]);
            }
            Direction::Out => {
                info!("Processing sent buffer {:?}", &buffer[..]);
            }
        }
    }
}
