use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, ImString};
use lobby_lib::LobbyEvent;

pub struct LoginScreen {
    username: ImString,
    password: ImString,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self {
            username: ImString::with_capacity(64),
            password: ImString::with_capacity(128),
        }
    }
}

impl Screen for LoginScreen {
    fn draw(
        &mut self,
        ui: &imgui::Ui,
        size: winit::dpi::PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        let window = imgui::Window::new(im_str!("Login"));
        let window_size = [260.0, 115.0];
        window
            .size(window_size, Condition::FirstUseEver)
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .position(
                [
                    size.width as f32 / 2.0 - window_size[0] / 2.0,
                    size.height as f32 / 2.0 - window_size[1] / 2.0,
                ],
                Condition::Always,
            )
            .build(&ui, || {
                ui.input_text(im_str!("Username"), &mut self.username)
                    .build();
                ui.input_text(im_str!("Password"), &mut self.password)
                    .password(true)
                    .build();
                ui.spacing();
                ui.spacing();
                if ui.button(im_str!("Login"), [50.0, 20.0]) {
                    action_sender.send(Action::Login {
                        username: self.username.to_string(),
                        password: self.password.to_string(),
                    });
                }
            });
    }
}
