#[macro_use]
extern crate lazy_static;

pub const PROTOCOL_VERSION: u16 = 1;
pub const APP_VERSION: u16 = 1;

pub mod net;
pub mod utils;

#[derive(Debug, Clone)]
pub enum LobbyEvent {
    Connected,
    Disconnected { message: String },
}
