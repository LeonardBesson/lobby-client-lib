use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, Ui};
use lobby_lib::net::structs::{LobbyInviteActionChoice, LobbyMember, UserProfile};
use lobby_lib::LobbyEvent;
use winit::dpi::PhysicalSize;

type Invite = (String, UserProfile);

struct Lobby {
    id: String,
    members: Vec<LobbyMember>,
    messages: Vec<String>,
}

pub struct LobbyScreen {
    invite: Option<Invite>,
    lobby: Option<Lobby>,
}

impl LobbyScreen {
    pub fn new() -> Self {
        Self {
            invite: None,
            lobby: None,
        }
    }

    fn update(&mut self, events: &[LobbyEvent]) {
        for event in events {
            match event {
                LobbyEvent::LobbyInvite { id, inviter } => {
                    self.invite = Some((id.clone(), inviter.clone()))
                }
                LobbyEvent::LobbyJoined { lobby_id } => {
                    self.lobby = Some(Lobby {
                        id: lobby_id.clone(),
                        members: Vec::new(),
                        messages: Vec::new(),
                    });
                }
                LobbyEvent::LobbyMemberUpdate { lobby_id, members } => {
                    let lobby = self
                        .lobby
                        .as_mut()
                        .expect("Received lobby update but no lobby is active");
                    assert_eq!(&lobby.id, lobby_id, "Received lobby update with wrong id");
                    lobby.members = members.clone();
                }
                LobbyEvent::LobbyLeft { lobby_id } => {
                    let lobby = self
                        .lobby
                        .as_ref()
                        .expect("Received lobby left but no lobby is active");
                    assert_eq!(&lobby.id, lobby_id, "Received lobby left with wrong id");
                    self.lobby = None;
                }
                _ => {}
            }
        }
    }
}

const MEMBER_ONLINE_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const MEMBER_OFFLINE_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.5];

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
        if let Some(lobby) = &self.lobby {
            imgui::Window::new(im_str!("Lobby"))
                .position([size.width as f32 - 400.0, 300.0], Condition::FirstUseEver)
                .size([400.0, 400.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.columns(2, im_str!("lobby columns"), true);
                    for member in &lobby.members {
                        let color = if member.is_online {
                            MEMBER_ONLINE_COLOR
                        } else {
                            MEMBER_OFFLINE_COLOR
                        };
                        ui.text_colored(color, &member.user_profile.display_name);
                    }
                    ui.next_column();
                    for message in &lobby.messages {
                        ui.text(message);
                    }
                });
        }
    }
}
