use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::net;
use crate::net::Message;
use crate::net::{packets, ErrorKind};
use crate::utils::byte_buffer::ByteBuffer;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PacketFlag {
    // First bit always on
    FixedHeader = 1 << 7,
    // Indicate 8 or 16 bits type
    ShortType = 1 << 6,
    // Indicate 8 or 24 bits size
    ShortSize = 1 << 5,
    // 5 bits left for future flags
}

#[derive(Debug, Copy, Clone)]
pub struct PacketInfo {
    pub packet_type: PacketType,
    pub name: &'static str,
    pub fixed_size: Option<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u16)]
pub enum PacketType {
    ClientInitRequest = 0,
    ClientInitResponse = 1,
}

pub struct Packet {
    pub flags: u8,
    pub packet_type: PacketType,
    pub data: ByteBuffer,
}

pub fn message_to_packet<'de, T: Message<'de>>(message: &T) -> net::Result<Packet> {
    let packet_data = Message::serialize_data(message).map_err(|err| {
        Box::new(ErrorKind::Serialize(format!(
            "Could not serialize packet {:?} data: {:?}",
            message.packet_type(),
            err
        )))
    })?;
    Ok(Packet::new(message.packet_type(), packet_data))
}

pub fn packet_to_message<'de, T: Message<'de>>(packet: &'de Packet) -> net::Result<T> {
    if !packets::has(packet.packet_type) {
        return Err(Box::new(ErrorKind::InvalidPacketType(packet.packet_type)));
    }
    Message::deserialize(&packet.data[..]).map_err(|err| {
        Box::new(ErrorKind::Deserialize(format!(
            "Could not deserialize packet type {:?}: {:?}",
            packet.packet_type, err
        )))
    })
}

impl Packet {
    pub fn new(packet_type: PacketType, data: Vec<u8>) -> Self {
        let packet_info = packets::get(packet_type);
        if let Some(fixed_size) = packet_info.fixed_size {
            assert_eq!(
                fixed_size,
                data.len(),
                "{}",
                format!(
                    "Attempt to create fixed size packet {:?} with wrong size",
                    packet_type
                )
            );
        }

        let mut flags = PacketFlag::FixedHeader as u8;
        if data.len() < 256 {
            flags |= PacketFlag::ShortSize as u8;
        }
        if (packet_type as u16) < 256 {
            flags |= PacketFlag::ShortType as u8;
        }

        Self {
            flags,
            packet_type,
            data: data.into(),
        }
    }

    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    pub fn short_type(&self) -> bool {
        (self.flags & PacketFlag::ShortType as u8) != 0
    }

    pub fn short_size(&self) -> bool {
        (self.flags & PacketFlag::ShortSize as u8) != 0
    }
}
