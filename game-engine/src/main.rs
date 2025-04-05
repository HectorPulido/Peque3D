extern crate mlua;
extern crate nalgebra as na;
extern crate piston_window;

mod camera3d;
mod input;
mod object3d;
mod rendering;
mod scripting;

use camera3d::Camera3d;
use input::InputSystem;
use rendering::Windows;
use scripting::LuaInt;

const CAMERA_SIZE_X: u32 = 800;
const CAMERA_SIZE_Y: u32 = 600;

fn main() -> mlua::Result<()> {
    let camera = Camera3d::new(
        60.0,
        (CAMERA_SIZE_X as f32) / (CAMERA_SIZE_Y as f32),
        0.1,
        1000.0,
    );

    let mut window = Windows::new(CAMERA_SIZE_X, CAMERA_SIZE_Y, "Rust 3D Motor", camera);

    let lua = LuaInt::new()?;
    let mut input_system = InputSystem::new();
    let mut last_time = std::time::Instant::now();

    while let Some(event) = window.window.next() {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_time).as_secs_f32();
        last_time = now;

        let inputs = input_system.get_inputs();
        lua.update(dt, &inputs)?;

        window.draw(&event, &mut lua.objects.borrow_mut());
    }

    Ok(())
}
