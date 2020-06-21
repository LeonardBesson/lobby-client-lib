use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, ImString, StyleColor, Ui};
use lobby_lib::net::structs::UserProfile;
use lobby_lib::LobbyEvent;
use regex::Regex;
use winit::dpi::PhysicalSize;

#[derive(Debug, Clone, PartialEq)]
pub enum TabKind {
    Empty,
    System,
    User(String),
}

#[derive(Debug, Clone)]
pub struct Tab {
    id: usize,
    kind: TabKind,
    lines: Vec<String>,
}

impl Tab {
    pub fn new(id: usize, kind: TabKind) -> Self {
        Self {
            id,
            kind,
            lines: Vec::new(),
        }
    }
}

pub struct ChatScreen {
    input: ImString,
    tabs: Vec<Tab>,
    selected_tab: Option<usize>,
}

impl ChatScreen {
    pub fn new() -> Self {
        Self {
            input: ImString::with_capacity(64),
            tabs: vec![Tab::new(0, TabKind::System)],
            selected_tab: Some(0),
        }
    }

    fn new_tab(&mut self, kind: TabKind) {
        self.tabs.push(Tab {
            id: self.tabs.len(),
            kind,
            lines: Vec::new(),
        });
    }

    fn get_tab(&self, id: usize) -> Option<&Tab> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    fn update_tab<L: Fn() -> String>(&mut self, kind: TabKind, line: L) -> bool {
        for tab in self.tabs.iter_mut() {
            match tab {
                Tab { kind: tab_king, lines, .. } if *tab_king == kind => {
                    lines.push(line());
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn print_tab(&mut self, ui: &Ui, id: usize) -> bool {
        for tab in &self.tabs {
            match tab {
                Tab { id: tab_id, kind, lines } if *tab_id == id => {
                    if *kind == TabKind::System {
                        print_lines(&ui, &lines, Some(RED));
                    } else {
                        print_lines(&ui, &lines, None);
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn new_user_tab(&mut self, from: &UserProfile, content: &str) {
        let id = self.tabs.len();
        self.tabs.push(Tab {
            id,
            kind: TabKind::User(from.user_tag.clone()),
            lines: vec![format!("[{}] {}", from.display_name, content)],
        });
        self.selected_tab = Some(id);
    }

    fn update(&mut self, events: &[LobbyEvent]) {
        for event in events {
            match event {
                LobbyEvent::NewPrivateMessage { from, content } => {
                    if !self.update_tab(TabKind::User(from.user_tag.clone()), || content.clone()) {
                        self.new_user_tab(from, content);
                    }
                }
                LobbyEvent::SystemNotification { content } => {
                    self.update_tab(TabKind::System, || content.clone());
                }
                _ => {}
            }
        }
    }
}

lazy_static! {
    static ref MESSAGE_REGEX: Regex = Regex::new(r"/w (\S+) (\w+)").unwrap();
}

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

fn print_lines(ui: &Ui, lines: &[String], color: Option<[f32; 4]>) {
    let style = color.map(|c| ui.push_style_color(StyleColor::Text, c));
    for line in lines {
        ui.text(line);
    }
    style.map(|s| s.pop(&ui));
}

impl Screen for ChatScreen {
    fn draw(
        &mut self,
        ui: &Ui,
        size: PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        self.update(events);

        let window = imgui::Window::new(im_str!("Chat"));
        window
            .size([400.0, 300.0], Condition::FirstUseEver)
            .position([5.0, size.height as f32 - 305.0], Condition::FirstUseEver)
            .build(&ui, || {
                let [width, height] = ui.window_size();
                for tab in &self.tabs {
                    let Tab { id, kind, .. } = tab;
                    let tab_id_string = id.to_string();
                    let button_text = match kind {
                        TabKind::Empty => tab_id_string.as_str(),
                        TabKind::System => "System",
                        TabKind::User(user_tag) => user_tag,
                    };
                    ui.same_line(0.0);
                    let id_token = ui.push_id(*id as i32);
                    if ui.button(&ImString::new(button_text), [0.0, 0.0]) {
                        self.selected_tab = Some(*id);
                    }
                    id_token.pop(&ui);
                }
                ui.same_line(0.0);
                if ui.button(im_str!("+"), [30.0, 0.0]) {
                    self.new_tab(TabKind::Empty);
                }

                if let Some(tab_id) = self.selected_tab {
                    self.print_tab(&ui, tab_id);
                }

                ui.set_cursor_pos([8.0, height - 30.0]);
                ui.push_item_width(width - 25.0);
                if ui
                    .input_text(im_str!(""), &mut self.input)
                    .enter_returns_true(true)
                    .resize_buffer(true)
                    .build()
                {
                    if let Some(cap) = MESSAGE_REGEX.captures(self.input.to_str()) {
                        let user_tag = cap[1].to_owned();
                        let content = cap[2].to_owned();
                        action_sender.send(Action::SendPrivateMessage { user_tag, content });
                    } else if let Some(tab_id) = self.selected_tab {
                        self.get_tab(tab_id).map(|tab| {
                            match &tab.kind {
                                TabKind::User(user_tag) => {
                                    let content = self.input.to_string();
                                    action_sender.send(Action::SendPrivateMessage {
                                        user_tag: user_tag.clone(),
                                        content,
                                    });
                                }
                                _ => {}
                            }
                        });
                    }
                    self.input.clear();
                }
            });
    }
}
