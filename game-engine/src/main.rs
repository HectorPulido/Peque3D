extern crate mlua;
extern crate nalgebra as na;
extern crate wgpu;
extern crate winit;

mod camera3d;
mod input;
mod object3d;
mod rendering;
mod scripting;
mod sound_system;

use camera3d::Camera3d;
use input::InputSystem;
use rendering::Renderer;
use scripting::LuaInt;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{WindowAttributes, WindowId},
};

const CAMERA_SIZE_X: u32 = 1600;
const CAMERA_SIZE_Y: u32 = 1200;
const WINDOWS_TITLE: &str = "Rust 3D Motor";

struct App {
    state: Option<Renderer>,
    last_time: std::time::Instant,
    input_system: InputSystem,
    lua: LuaInt,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: None,
            last_time: std::time::Instant::now(),
            input_system: InputSystem::new(),
            lua: LuaInt::new().unwrap(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize everything here
        let mut attrs = WindowAttributes::default();
        attrs.title = WINDOWS_TITLE.into();
        attrs.inner_size = Some(winit::dpi::Size::Physical(PhysicalSize::new(
            CAMERA_SIZE_X,
            CAMERA_SIZE_Y,
        )));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let camera = Camera3d::new(
            60.0,
            (CAMERA_SIZE_X as f32) / (CAMERA_SIZE_Y as f32),
            0.1,
            1000.0,
        );

        let state = pollster::block_on(Renderer::new(
            window.clone(),
            CAMERA_SIZE_X,
            CAMERA_SIZE_Y,
            camera,
        ));
        self.state = Some(state);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    // This is like the main loop
                    let now = std::time::Instant::now();
                    let dt = now.duration_since(self.last_time).as_secs_f32();
                    self.last_time = now;

                    let inputs = self.input_system.get_inputs();
                    if let Err(e) = self.lua.update(dt, &inputs) {
                        eprintln!("Error en scripting: {:?}", e);
                    }

                    if let Err(e) = state.draw(&mut self.lua.objects.borrow_mut()) {
                        eprintln!("Error en render: {:?}", e);
                    }
                    state.get_window().request_redraw();
                }
                WindowEvent::Resized(size) => state.resize(size),
                _ => {}
            }
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
