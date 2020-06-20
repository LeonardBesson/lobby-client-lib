use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, ImString, StyleVar, Ui};

use lobby_lib::LobbyEvent;
use winit::dpi::PhysicalSize;

pub struct FriendListScreen {
    user_tag_input: ImString,
}

impl FriendListScreen {
    pub fn new() -> Self {
        Self {
            user_tag_input: ImString::with_capacity(128),
        }
    }
}

impl Screen for FriendListScreen {
    fn draw(
        &mut self,
        ui: &Ui,
        size: PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        let window = imgui::Window::new(im_str!("Friends"));
        window
            .size([260.0, 350.0], Condition::FirstUseEver)
            .position([size.width as f32 - 320.0, 5.0], Condition::FirstUseEver)
            .build(&ui, || {
                imgui::ChildWindow::new("Add Friend Input").build(&ui, || {
                    ui.text("User Tag:");
                    ui.same_line(0.0);
                    let width = ui.push_item_width(120.0);
                    ui.input_text(im_str!(""), &mut self.user_tag_input).build();
                    ui.same_line(0.0);
                    width.pop(&ui);
                    if ui.button(im_str!("Add"), [0.0, 0.0]) {
                        action_sender.send(Action::AddFriend {
                            user_tag: self.user_tag_input.to_string(),
                        });
                    }
                })
            });
    }
}
