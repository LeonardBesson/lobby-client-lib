use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Instant;

use imgui::*;
use imgui_winit_support::WinitPlatform;
use winit::window::Window;

use crate::renderer::Renderer;
use crate::time::Time;
use crate::ui::screens::Screen;

pub mod screens;

pub struct Ui {
    imgui: imgui::Context,
    platform: WinitPlatform,
    renderer: Option<imgui_wgpu::Renderer>,
    mouse_cursor: Option<MouseCursor>,
    current_screen: Option<Box<dyn Screen>>,
}

impl Ui {
    pub fn new() -> Self {
        let mut imgui = imgui::Context::create();
        let platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        Self {
            imgui,
            platform,
            renderer: None,
            mouse_cursor: None,
            current_screen: None,
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

    pub fn push_screen(&mut self, screen: Box<dyn Screen>) {
        self.current_screen = Some(screen);
    }

    pub fn draw(
        &mut self,
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

        if let Some(screen) = self.current_screen.as_mut() {
            screen.draw(&ui, window.inner_size());
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
