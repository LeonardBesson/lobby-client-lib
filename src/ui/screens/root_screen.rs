use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, Ui};
use lobby_lib::LobbyEvent;
use winit::dpi::PhysicalSize;

pub struct RootScreen;

impl Screen for RootScreen {
    fn draw(
        &mut self,
        ui: &Ui,
        size: PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        let window = imgui::Window::new(im_str!("Exit Button Window"));
        window
            .movable(false)
            .no_decoration()
            .position([(size.width - 57) as f32, 0.0], Condition::FirstUseEver)
            .build(&ui, || {
                if ui.button(im_str!("Exit"), [0.0, 0.0]) {
                    action_sender.send(Action::Exit);
                }
            });
    }
}
