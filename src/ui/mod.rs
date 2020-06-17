use crate::application::Action;
use crate::renderer::Renderer;
use crate::time::Time;
use crate::ui::screens::{Screen, ScreenToken};
use crossbeam_channel::Sender;
use imgui::*;
use imgui_winit_support::WinitPlatform;
use lobby_lib::LobbyEvent;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Instant;
use winit::window::Window;

pub mod screens;

pub struct Ui {
    imgui: imgui::Context,
    platform: WinitPlatform,
    renderer: Option<imgui_wgpu::Renderer>,
    mouse_cursor: Option<MouseCursor>,
    action_sender: Sender<Action>,
    screen_indexes: HashMap<String, usize>,
    screens: Vec<Option<Box<dyn Screen>>>,
}

impl Ui {
    pub fn new(action_sender: Sender<Action>) -> Self {
        let mut imgui = imgui::Context::create();
        let platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        Self {
            imgui,
            platform,
            renderer: None,
            mouse_cursor: None,
            action_sender,
            screen_indexes: HashMap::new(),
            screens: Vec::new(),
        }
    }

    pub fn init(&mut self, window: &Window, renderer: &mut Renderer) {
        self.platform.attach_window(
            self.imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
        self.imgui.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();
        let font_size = (15.0 * hidpi_factor) as f32;
        self.imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        self.imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let renderer = imgui_wgpu::Renderer::new(
            &mut self.imgui,
            &renderer.device,
            &mut renderer.queue,
            renderer.sc_desc.format,
            Some(clear_color),
        );

        self.renderer = Some(renderer);
    }

    pub fn handle_event<T>(&mut self, window: &Window, event: &winit::event::Event<T>) {
        self.platform
            .handle_event(self.imgui.io_mut(), window, event);
    }

    pub fn push_screen(&mut self, token: ScreenToken, screen: Box<dyn Screen>) -> usize {
        let z_order = self.screens.len();
        self.insert_screen(token, z_order, screen)
    }

    pub fn insert_screen(
        &mut self,
        token: ScreenToken,
        z_order: usize,
        screen: Box<dyn Screen>,
    ) -> usize {
        if z_order > self.screens.len() {
            self.screens.resize_with(z_order, || None);
        }
        self.screen_indexes.insert(token.to_owned(), z_order);
        self.screens.insert(z_order, Some(screen));
        z_order
    }

    pub fn remove_screen(&mut self, token: ScreenToken) -> Option<Box<dyn Screen>> {
        let token = token.to_owned();
        if let Some(z_order) = self.screen_indexes.get(&token).copied() {
            self.screen_indexes.remove(&token);
            return self.screens.remove(z_order);
        }
        None
    }

    pub fn replace_screen(
        &mut self,
        old_token: ScreenToken,
        new_token: ScreenToken,
        new_screen: Box<dyn Screen>,
    ) -> Option<Box<dyn Screen>> {
        if let Some(z_order) = self.screen_indexes.get(&old_token.to_owned()).copied() {
            self.screen_indexes.insert(new_token.to_owned(), z_order);
            let old_screen = self.screens.remove(z_order);
            self.screens.insert(z_order, Some(new_screen));
            return old_screen;
        }
        None
    }

    pub fn draw(
        &mut self,
        events: &[LobbyEvent],
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::SwapChainOutput,
        window: &Window,
        time: &Time,
    ) {
        self.imgui.io_mut().delta_time = time.delta;

        self.platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare frame");
        let ui = self.imgui.frame();

        for screen in self.screens.iter_mut() {
            if let Some(screen) = screen {
                screen.draw(&ui, window.inner_size(), events, &self.action_sender);
            }
        }

        if self.mouse_cursor != ui.mouse_cursor() {
            self.mouse_cursor = ui.mouse_cursor();
            self.platform.prepare_render(&ui, &window);
        }
        self.renderer
            .as_mut()
            .expect("ui::init() was never called")
            .render(ui.render(), device, encoder, &frame.view)
            .expect("Rendering failed");
    }
}
