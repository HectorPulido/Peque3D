use device_query::{DeviceQuery, DeviceState, Keycode};

pub struct Input {
    pub input: Vec<Keycode>,
    pub just_pressed: Vec<Keycode>,
    pub just_released: Vec<Keycode>,
}

pub struct InputSystem {
    device_state: DeviceState,
    last_time_input: Vec<Keycode>,
}

impl InputSystem {
    pub fn new() -> Self {
        let device_state = DeviceState::new();
        let last_time_input = Vec::new();
        InputSystem {
            device_state,
            last_time_input,
        }
    }

    pub fn get_inputs(&mut self) -> Input {
        let input: Vec<Keycode> = self.device_state.get_keys();
        let mut just_pressed = Vec::new();
        for key in input.iter() {
            if !self.last_time_input.contains(key) {
                just_pressed.push(key.clone());
            }
        }
        let mut just_released = Vec::new();
        for key in self.last_time_input.iter() {
            if !input.contains(key) {
                just_released.push(key.clone());
            }
        }

        self.last_time_input = input.clone();

        Input {
            input: input.clone(),
            just_pressed,
            just_released,
        }
    }
}
