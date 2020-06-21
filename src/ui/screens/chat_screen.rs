use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, ImString, StyleColor, Ui};
use lobby_lib::LobbyEvent;
use regex::Regex;
use winit::dpi::PhysicalSize;

#[derive(Debug, Clone, PartialEq)]
enum TabKind {
    Empty,
    System,
    User(String),
}

#[derive(Debug, Clone)]
pub struct Tab(usize, TabKind);

pub struct ChatScreen {
    input: ImString,
    tabs: Vec<(Tab, Vec<String>)>,
    selected_tab: Option<usize>,
}

impl ChatScreen {
    pub fn new() -> Self {
        Self {
            input: ImString::with_capacity(64),
            tabs: vec![(Tab(0, TabKind::System), Vec::new())],
            selected_tab: Some(0),
        }
    }

    fn new_tab(&mut self) {
        self.tabs
            .push((Tab(self.tabs.len(), TabKind::Empty), Vec::new()));
    }

    fn update(&mut self, events: &[LobbyEvent]) {
        for event in events {
            match event {
                LobbyEvent::NewPrivateMessage { from, content } => {
                    for (tab, lines) in self.tabs.iter_mut() {
                        match tab {
                            Tab(tab_id, TabKind::User(user_tag)) if user_tag == &from.user_tag => {
                                lines.push(content.clone());
                                return;
                            }
                            _ => {}
                        }
                    }
                    let tab_id = self.tabs.len();
                    self.tabs.push((
                        Tab(tab_id, TabKind::User(from.user_tag.clone())),
                        vec![content.clone()],
                    ));
                    self.selected_tab = Some(tab_id);
                }
                LobbyEvent::SystemNotification { content } => {
                    for (tab, lines) in self.tabs.iter_mut() {
                        match tab {
                            Tab(_, TabKind::System) => {
                                lines.push(content.clone());
                                return;
                            }
                            _ => {}
                        }
                    }
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
    let style = color.map(|c| ui.push_style_color(StyleColor::Text, RED));
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
                for (tab, _) in &self.tabs {
                    let Tab(tab_id, tab_kind) = tab;
                    let tab_id_string = tab_id.to_string();
                    let button_text = match tab_kind {
                        TabKind::Empty => tab_id_string.as_str(),
                        TabKind::System => "System",
                        TabKind::User(user_tag) => user_tag,
                    };
                    ui.same_line(0.0);
                    let id = ui.push_id(*tab_id as i32);
                    if ui.button(&ImString::new(button_text), [0.0, 0.0]) {
                        self.selected_tab = Some(*tab_id);
                        println!("selected tab: {:?}", self.selected_tab);
                    }
                    id.pop(&ui);
                }
                ui.same_line(0.0);
                if ui.button(im_str!("+"), [30.0, 0.0]) {
                    self.new_tab();
                }

                if let Some(tab_id) = self.selected_tab {
                    let (Tab(_, kind), lines) = self
                        .tabs
                        .iter()
                        .find(|(Tab(id, _), _)| tab_id == *id)
                        .unwrap();

                    if *kind == TabKind::System {
                        print_lines(&ui, &lines, Some(RED));
                    } else {
                        print_lines(&ui, &lines, None);
                    }
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
                        self.input.clear();
                    }
                }
            });
    }
}
