use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, ImString, StyleVar, Ui};

use lobby_lib::net::packets::PacketType::FriendRequestAction;
use lobby_lib::net::structs::{Friend, FriendRequest, FriendRequestActionChoice, UserProfile};
use lobby_lib::LobbyEvent;
use winit::dpi::PhysicalSize;

pub struct FriendListScreen {
    user_tag_input: ImString,
    pending_as_inviter: Vec<FriendRequest>,
    pending_as_invitee: Vec<FriendRequest>,
    friend_list: Vec<Friend>,
}

impl FriendListScreen {
    pub fn new() -> Self {
        Self {
            user_tag_input: ImString::with_capacity(128),
            pending_as_inviter: Vec::new(),
            pending_as_invitee: Vec::new(),
            friend_list: Vec::new(),
        }
    }

    pub fn update(&mut self, events: &[LobbyEvent]) {
        for event in events {
            match event {
                LobbyEvent::FriendRequestsUpdated {
                    as_inviter,
                    as_invitee,
                } => {
                    self.pending_as_inviter = as_inviter.to_vec();
                    self.pending_as_invitee = as_invitee.to_vec();
                }
                LobbyEvent::FriendListUpdated { friend_list } => {
                    self.friend_list = friend_list.to_vec();
                }
                _ => {}
            }
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
        self.update(events);

        let window = imgui::Window::new(im_str!("Friends"));
        window
            .size([260.0, 350.0], Condition::FirstUseEver)
            .position([size.width as f32 - 320.0, 5.0], Condition::FirstUseEver)
            .build(&ui, || {
                // Input
                let input_group = ui.begin_group();
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
                input_group.end(&ui);
                ui.separator();
                ui.spacing();
                ui.spacing();
                ui.spacing();

                // Invitee
                let invitee_requests = self
                    .pending_as_invitee
                    .iter()
                    .filter(|r| r.state == "pending")
                    .collect::<Vec<_>>();
                if !invitee_requests.is_empty() {
                    let invitee_group = ui.begin_group();
                    ui.text("Pending Requests:");
                    ui.separator();
                    for request in invitee_requests {
                        ui.indent();
                        ui.text(format!(
                            "{} ({})",
                            &request.user_profile.display_name, &request.user_profile.user_tag
                        ));
                        if ui.button(im_str!("Accept"), [0.0, 0.0]) {
                            action_sender.send(Action::FriendRequestAction {
                                request_id: request.id.clone(),
                                action: FriendRequestActionChoice::Accept,
                            });
                        }
                        ui.same_line(0.0);
                        if ui.button(im_str!("Decline"), [0.0, 0.0]) {
                            action_sender.send(Action::FriendRequestAction {
                                request_id: request.id.clone(),
                                action: FriendRequestActionChoice::Decline,
                            });
                        }
                        ui.unindent();
                        ui.separator();
                    }
                    invitee_group.end(&ui);
                    ui.spacing();
                    ui.spacing();
                    ui.spacing();
                }

                // Inviter
                let inviter_requests = self
                    .pending_as_inviter
                    .iter()
                    .filter(|r| r.state == "pending")
                    .collect::<Vec<_>>();
                if !inviter_requests.is_empty() {
                    let inviter_group = ui.begin_group();
                    ui.text("Sent Requests:");
                    ui.separator();
                    for request in inviter_requests {
                        ui.indent();
                        ui.text(format!(
                            "{} ({})",
                            &request.user_profile.display_name, &request.user_profile.user_tag
                        ));
                        ui.unindent();
                        ui.separator();
                    }
                    inviter_group.end(&ui);
                }

                let (online_friends, offline_friends): (Vec<&Friend>, Vec<&Friend>) =
                    self.friend_list.iter().partition(|f| f.is_online);
                let online_friends_group = ui.begin_group();
                ui.text("Online Friends:");
                ui.separator();
                for friend in online_friends {
                    ui.indent();
                    ui.text(format!(
                        "{} ({})",
                        &friend.user_profile.display_name, &friend.user_profile.user_tag
                    ));
                    ui.same_line(0.0);
                    if ui.button(im_str!("Remove"), [0.0, 0.0]) {
                        action_sender.send(Action::RemoveFriend {
                            user_tag: friend.user_profile.user_tag.clone(),
                        });
                    }
                    ui.unindent();
                    ui.separator();
                }
                online_friends_group.end(&ui);

                let offline_friends_group = ui.begin_group();
                ui.text("Offline Friends:");
                ui.separator();
                for friend in offline_friends {
                    ui.indent();
                    ui.text(format!(
                        "{} ({})",
                        &friend.user_profile.display_name, &friend.user_profile.user_tag
                    ));
                    ui.same_line(0.0);
                    if ui.button(im_str!("Remove"), [0.0, 0.0]) {
                        action_sender.send(Action::RemoveFriend {
                            user_tag: friend.user_profile.user_tag.clone(),
                        });
                    }
                    ui.unindent();
                    ui.separator();
                }
                offline_friends_group.end(&ui);
            });
    }
}
