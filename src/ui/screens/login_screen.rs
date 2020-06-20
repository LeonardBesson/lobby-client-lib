use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, ImString};
use lobby_lib::LobbyEvent;

pub struct LoginScreen {
    email: ImString,
    password: ImString,
}

impl LoginScreen {
    pub fn new() -> Self {
        let mut email = ImString::with_capacity(64);
        email.push_str("dev@lobby.com");
        let mut password = ImString::with_capacity(128);
        password.push_str("admin");
        Self { email, password }
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
                    size.width as f32 / 2.0 - window_size[0] / 2.0 + 30.0,
                    size.height as f32 / 2.0 - window_size[1] / 2.0,
                ],
                Condition::Always,
            )
            .build(&ui, || {
                ui.input_text(im_str!("Email"), &mut self.email).build();
                ui.input_text(im_str!("Password"), &mut self.password)
                    .password(true)
                    .build();
                ui.spacing();
                ui.spacing();
                if ui.button(im_str!("Login"), [50.0, 20.0]) {
                    action_sender.send(Action::Login {
                        email: self.email.to_string(),
                        password: self.password.to_string(),
                    });
                }
            });
    }
}
