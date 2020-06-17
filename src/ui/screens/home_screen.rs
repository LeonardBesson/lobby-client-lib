use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, Ui};
use lobby_lib::LobbyEvent;
use winit::dpi::PhysicalSize;

pub struct HomeScreen;

impl Screen for HomeScreen {
    fn draw(
        &mut self,
        ui: &Ui,
        size: PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        // TODO:
    }
}
