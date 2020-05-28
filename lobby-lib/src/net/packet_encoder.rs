use std::collections::VecDeque;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::net::packet::{Packet, PacketFlag};
use crate::net::packets;
use crate::utils::byte_buffer::ByteBuffer;

const MAX_PACKET_HEADER_SIZE: usize = 22;

pub struct PacketEncoder {
    pub target_size: usize,
    pub buffers: VecDeque<ByteBuffer>,
    pub packets: VecDeque<Packet>,
}

impl PacketEncoder {
    pub fn new(target_size: usize) -> Self {
        Self {
            target_size,
            buffers: VecDeque::new(),
            packets: VecDeque::new(),
        }
    }

    pub fn add_packet(&mut self, packet: Packet) {
        self.packets.push_back(packet);
    }

    pub fn next_buffer(&mut self) -> Option<ByteBuffer> {
        if !self.buffers.is_empty() {
            return self.buffers.pop_front();
        }

        self.pack(self.target_size);
        return self.buffers.pop_front();
    }

    fn pack(&mut self, target_size: usize) {
        if self.packets.is_empty() {
            return;
        }

        let mut packet_count = 0;
        let mut result: Vec<u8> = Vec::with_capacity(target_size);
        while let Some(packet) = self.packets.pop_front() {
            if !result.is_empty()
                && packet.data_size() + MAX_PACKET_HEADER_SIZE + result.len() > target_size
            {
                break;
            }
            if !packets::has(packet.packet_type) {
                panic!("Packet type {:?} not registered", packet.packet_type);
            }
            let packet_info = packets::get(packet.packet_type);

            let flags_offset = result.len();

            // Flags
            result.write_u8(packet.flags);

            // Type
            if packet.short_type() {
                result.write_u8(packet.packet_type as u8);
                result[flags_offset] |= PacketFlag::ShortType as u8;
            } else {
                result.write_u16::<BigEndian>(packet.packet_type as u16);
            }

            // Size
            if let None = packet_info.fixed_size {
                if packet.data_size() < 256 {
                    result.write_u8(packet.data_size() as u8);
                    result[flags_offset] |= PacketFlag::ShortSize as u8;
                } else {
                    result.write_u24::<BigEndian>(packet.data_size() as u32);
                    result[flags_offset] &= !(PacketFlag::ShortSize as u8);
                }
            }

            // Data
            if packet.data_size() > 0 {
                result.write_all(&packet.data);
            }

            packet_count += 1;
        }

        if packet_count > 0 {
            println!(
                "Packed {} packet(s) into buffer ({} bytes)",
                packet_count,
                result.len()
            )
        }
        self.buffers.push_back(result.into());
    }
}

#[cfg(test)]
mod tests {
    use crate::net::packet::{Packet, PacketType};
    use crate::net::packet_encoder::PacketEncoder;

    #[test]
    fn single_packet() {
        let mut encoder = PacketEncoder::new(256);
        encoder.add_packet(Packet::new(PacketType::ClientInitRequest, vec![1; 25]));
        let buffer = encoder.next_buffer();
        assert!(buffer.is_some());
        assert_eq!(buffer.unwrap().len(), 3 + 25);
    }

    #[test]
    fn size_flag() {
        let mut encoder = PacketEncoder::new(1024);
        encoder.add_packet(Packet::new(PacketType::ClientInitRequest, vec![1; 255]));
        let buffer = encoder.next_buffer();
        assert!(buffer.is_some());
        assert_eq!(buffer.unwrap().len(), 3 + 255);

        encoder.add_packet(Packet::new(PacketType::ClientInitRequest, vec![1; 256]));
        let buffer = encoder.next_buffer();
        assert!(buffer.is_some());
        assert_eq!(buffer.unwrap().len(), 5 + 256);
    }

    #[test]
    fn multiple_packets() {
        let mut encoder = PacketEncoder::new(256);
        encoder.add_packet(Packet::new(PacketType::ClientInitRequest, vec![1; 25]));
        encoder.add_packet(Packet::new(PacketType::ClientInitRequest, vec![1; 30]));
        let buffer = encoder.next_buffer();
        assert!(buffer.is_some());
        assert_eq!(buffer.unwrap().len(), 3 + 25 + 3 + 30);
    }
}
