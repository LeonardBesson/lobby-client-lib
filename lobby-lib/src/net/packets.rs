use crate::net::packet::PacketInfo;
use crate::net::structs::*;
use log::info;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

const MAX_PACKET_TYPES: usize = 500;

macro_rules! declare_packets {
    ($
        ($struct:ident {
            $($field:ident:$type:ty)*
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
        message: String
    }
    PacketInit {
        protocol_version: u16
        app_version: u16
    }
    AuthenticationRequest {
        email: String
        password: String
    }
    AuthenticationResponse {
        error_code: Option<String>
        session_token: Option<String>
        user_profile: Option<UserProfile>
    }
    PacketPing {
        id: String
        peer_time: u64
    }
    PacketPong {
        id: String
        peer_time: u64
    }
    AddFriendRequest {
        user_tag: String
    }
    AddFriendRequestResponse {
        user_tag: String
        error_code: Option<String>
    }
    FriendRequestAction {
        request_id: String
        action: crate::net::structs::FriendRequestAction
    }
    FriendRequestActionResponse {
        request_id: String
        error_code: Option<String>
    }
    FetchPendingFriendRequests {}
    FetchPendingFriendRequestsResponse {
        pending_as_inviter: Vec<FriendRequest>
        pending_as_invitee: Vec<FriendRequest>
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
    AuthenticationRequest = 2,
    AuthenticationResponse = 3,
    PacketPing = 4,
    PacketPong = 5,
    AddFriendRequest = 6,
    AddFriendRequestResponse = 7,
    FriendRequestAction = 8,
    FriendRequestActionResponse = 9,
    FetchPendingFriendRequests = 10,
    FetchPendingFriendRequestsResponse = 11,

    Last,
}

fn init_packets(types: &mut [Option<PacketInfo>; packet_count()]) {
    FatalError::register(types);
    PacketInit::register(types);
    AuthenticationRequest::register(types);
    AuthenticationResponse::register(types);
    PacketPing::register(types);
    PacketPong::register(types);
    AddFriendRequest::register(types);
    AddFriendRequestResponse::register(types);
    FriendRequestAction::register(types);
    FriendRequestActionResponse::register(types);
    FetchPendingFriendRequests::register(types);
    FetchPendingFriendRequestsResponse::register(types);
}

pub fn init() {
    info!(
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
