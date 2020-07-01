use crate::application::Action;
use crossbeam_channel::Sender;
use lobby_lib::LobbyEvent;

pub mod chat_screen;
pub mod events_screen;
pub mod friend_list_screen;
pub mod home_screen;
pub mod lobby_screen;
pub mod login_screen;
pub mod root_screen;

pub trait Screen {
    fn draw(
        &mut self,
        ui: &imgui::Ui,
        size: winit::dpi::PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    );
}
