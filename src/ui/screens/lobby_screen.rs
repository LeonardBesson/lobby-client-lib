use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, Ui};
use lobby_lib::net::structs::{LobbyInviteActionChoice, UserProfile};
use lobby_lib::LobbyEvent;
use winit::dpi::PhysicalSize;

type Invite = (String, UserProfile);

pub struct LobbyScreen {
    invite: Option<Invite>,
    // TODO: display lobby when joined
}

impl LobbyScreen {
    pub fn new() -> Self {
        Self { invite: None }
    }

    fn update(&mut self, events: &[LobbyEvent]) {
        for event in events {
            match event {
                LobbyEvent::LobbyInvite { id, inviter } => {
                    self.invite = Some((id.clone(), inviter.clone()))
                }
                _ => {}
            }
        }
    }
}

impl Screen for LobbyScreen {
    fn draw(
        &mut self,
        ui: &Ui,
        size: PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        self.update(events);
        let mut triggered_action = false;
        if let Some((id, inviter)) = &self.invite {
            imgui::Window::new(im_str!("Lobby invite"))
                .position([size.width as f32 - 300.0, 400.0], Condition::FirstUseEver)
                .no_decoration()
                .resizable(false)
                .build(&ui, || {
                    ui.text(format!(
                        "Lobby invite from {} ({})",
                        &inviter.display_name, &inviter.user_tag
                    ));
                    if ui.button(im_str!("Accept"), [0.0, 0.0]) {
                        action_sender.send(Action::LobbyInviteAction {
                            invite_id: id.clone(),
                            action: LobbyInviteActionChoice::Accept,
                        });
                        triggered_action = true;
                    }
                    ui.same_line(0.0);
                    if ui.button(im_str!("Decline"), [0.0, 0.0]) {
                        action_sender.send(Action::LobbyInviteAction {
                            invite_id: id.clone(),
                            action: LobbyInviteActionChoice::Decline,
                        });
                        triggered_action = true;
                    }
                });
        }
        if triggered_action {
            self.invite = None;
        }
    }
}
