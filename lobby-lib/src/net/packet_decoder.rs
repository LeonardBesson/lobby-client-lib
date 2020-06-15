use crate::net::packet::{Packet, PacketFlag};
use crate::net::packets;
use crate::net::packets::PacketType;
use crate::utils::byte_buffer::ByteBuffer;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};
use bytes::buf::BufExt;
use bytes::{Buf, BufMut, BytesMut};
use log::debug;
use num_traits::FromPrimitive;
use std::collections::VecDeque;
use std::io::Write;

pub struct PacketDecoder {
    stream: BytesMut,
}

impl PacketDecoder {
    pub fn new() -> Self {
        Self {
            stream: BytesMut::with_capacity(8 * 1024),
        }
    }

    pub fn push_buffer(&mut self, buffer: ByteBuffer) {
        self.stream.put(&buffer[..]);
    }

    pub fn next_packet(&mut self) -> Option<Packet> {
        if self.stream.remaining() < 1 {
            return None;
        }

        let flags = self.stream[0];
        if flags & PacketFlag::FixedHeader as u8 == 0 {
            panic!("Internal stream in invalid state. Aborting.")
        }

        // Header
        let mut header_size = 1;
        if flags & PacketFlag::ShortType as u8 != 0 {
            header_size += 1;
        } else {
            header_size += 2;
        };

        if flags & PacketFlag::ShortSize as u8 != 0 {
            header_size += 1;
        } else {
            header_size += 3;
        };

        if self.stream.remaining() < header_size {
            return None;
        }

        // Data
        let header = &self.stream[0..header_size];
        let flags = header[0];
        let mut offset = 1;

        let packet_type;
        if flags & PacketFlag::ShortType as u8 != 0 {
            packet_type = header[offset] as u16;
            offset += 1;
        } else {
            packet_type = byteorder::BigEndian::read_u16(&header[offset..offset + 2]);
            offset += 2;
        };

        let packet_type = PacketType::from_u16(packet_type)
            .expect(format!("Unknown packet type {}", packet_type).as_str());

        let packet_info = packets::get(packet_type);

        let data_size;
        if let Some(fixed_size) = packet_info.fixed_size {
            data_size = fixed_size;
        } else {
            if flags & PacketFlag::ShortSize as u8 != 0 {
                data_size = header[offset] as usize;
                offset += 1;
            } else {
                data_size = byteorder::BigEndian::read_u24(&header[offset..offset + 3]) as usize;
                offset += 3;
            };
        }

        // Done
        if self.stream.remaining() < header_size + data_size {
            return None;
        }

        let header = self.stream.split_to(header_size);
        let data = self.stream.split_to(data_size);

        debug!(
            "Decoded new packet. Header: {:?} (Flags: {}, Type: {:?}, Data Size: {}) Data: {:?}",
            header, flags, packet_type, data_size, data
        );
        Some(Packet::new(packet_type, data.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use crate::net::packet::{Packet, PacketFlag};
    use crate::net::packet_decoder::PacketDecoder;
    use crate::net::packet_encoder::PacketEncoder;
    use crate::net::packets::PacketType;

    #[test]
    fn single_packet() {
        let mut encoder = PacketEncoder::new(256);
        encoder.add_packet(Packet::new(PacketType::PacketInit, vec![1; 25]));
        let buffer = encoder.next_buffer().unwrap();
        assert_eq!(buffer.len(), 3 + 25);

        let mut decoder = PacketDecoder::new();
        decoder.push_buffer(buffer);
        let packet = decoder.next_packet().unwrap();
        assert!(packet.short_type());
        assert!(packet.short_size());
        assert_eq!(packet.packet_type, PacketType::PacketInit);
        assert_eq!(&packet.data[..], &[1; 25]);

        assert!(decoder.next_packet().is_none());
    }

    #[test]
    fn multiple_packets() {
        let mut encoder = PacketEncoder::new(256);
        encoder.add_packet(Packet::new(PacketType::PacketInit, vec![1; 25]));
        encoder.add_packet(Packet::new(PacketType::PacketInit, vec![1; 75]));
        let buffer = encoder.next_buffer().unwrap();
        assert_eq!(buffer.len(), 3 + 25 + 3 + 75);

        let mut decoder = PacketDecoder::new();
        decoder.push_buffer(buffer);
        let packet = decoder.next_packet().unwrap();
        assert!(packet.short_type());
        assert!(packet.short_size());
        assert_eq!(packet.packet_type, PacketType::PacketInit);
        assert_eq!(&packet.data[..], &[1; 25]);

        let packet = decoder.next_packet().unwrap();
        assert!(packet.short_type());
        assert!(packet.short_size());
        assert_eq!(packet.packet_type, PacketType::PacketInit);
        assert_eq!(&packet.data[..], &vec![1; 75][..]);

        assert!(decoder.next_packet().is_none());
    }
}
