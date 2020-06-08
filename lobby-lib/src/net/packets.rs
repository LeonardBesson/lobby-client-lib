use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::net::packet::PacketInfo;

const MAX_PACKET_TYPES: usize = 500;

macro_rules! declare_packets {
    ($
        ($struct:ident {
            $($field:ident:$type:ty),*,
        })
    +) => {
        $(
            #[derive(Debug, Serialize, Deserialize)]
            pub struct $struct {
                $(
                    pub $field: $type,
                )*
            }

            impl $struct {
                pub const TYPE: PacketType = PacketType::$struct;
                pub const INFO: PacketInfo = PacketInfo {
                    packet_type: Self::TYPE,
                    name: stringify!($struct),
                    fixed_size: None,
                };

                pub fn register(packets: &mut [Option<PacketInfo>; packet_count()]) {
                    assert!((Self::TYPE as usize) < MAX_PACKET_TYPES, "Max number of packets reached");
                    packets[Self::TYPE as usize] = Some(Self::INFO);
                }
            }

            impl crate::net::Message<'_> for $struct {
                fn packet_type(&self) -> PacketType {
                    Self::TYPE
                }
                fn packet_info(&self) -> PacketInfo {
                    Self::INFO
                }
            }
        )+
    };
}

declare_packets! {
    FatalError {
        message: String,
    }
    PacketInit {
        protocol_version: u16,
        app_version: u16,
    }
}

lazy_static! {
    static ref PACKET_INFOS: [Option<PacketInfo>; packet_count()] = {
        let mut types = [None; packet_count()];
        init_packets(&mut types);
        types
    };
}

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u16)]
pub enum PacketType {
    FatalError = 0,
    PacketInit = 1,

    Last,
}

fn init_packets(types: &mut [Option<PacketInfo>; packet_count()]) {
    FatalError::register(types);
    PacketInit::register(types);
}

pub fn init() {
    println!(
        "Initialized {} packet types",
        PACKET_INFOS.iter().filter(|info| info.is_some()).count()
    );
}

pub const fn packet_count() -> usize {
    PacketType::Last as usize
}

pub fn has(packet_type: PacketType) -> bool {
    (packet_type as usize) < packet_count() && PACKET_INFOS[packet_type as usize].is_some()
}

pub fn get(packet_type: PacketType) -> PacketInfo {
    PACKET_INFOS[packet_type as usize].expect("Packet type {:?} is not registered!")
}
