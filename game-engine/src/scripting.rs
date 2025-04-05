use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use na::Vector3;
use std::{cell::RefCell, rc::Rc};

use super::input::Input;
use super::object3d::Object3d;

pub struct LuaInt {
    lua: Lua,
    pub objects: Rc<RefCell<Vec<Object3d>>>,
    pub pending_objects: Rc<RefCell<Vec<Object3d>>>,
}

impl LuaInt {
    pub fn new() -> LuaResult<Self> {
        // Usa Rc para poder compartir el vector de objetos
        let objects = Rc::new(RefCell::new(vec![
            Object3d::new("model/aircraft.obj", Vector3::new(0.0, 0.0, 5.0), 0.0),
        ]));
        let pending_objects = Rc::new(RefCell::new(Vec::new()));
        let lua = Lua::new();

        lua.globals().set(
            "print",
            lua.create_function(|_, args: MultiValue| {
                let output: Vec<String> = args
                    .into_iter()
                    .map(|val| match val {
                        Value::String(s) => Ok(s.to_str()?.to_string()),
                        other => Ok(format!("{:?}", other)),
                    })
                    .collect::<LuaResult<Vec<String>>>()?;
                println!("{}", output.join("\t"));
                Ok(())
            })?,
        )?;

        {
            let pending_objects_clone = Rc::clone(&pending_objects);
            lua.globals().set(
                "new_model",
                lua.create_function_mut(
                    move |_, (model_path, pos_table, rotation): (String, Table, f32)| {
                        let x: f32 = pos_table.get("x")?;
                        let y: f32 = pos_table.get("y")?;
                        let z: f32 = pos_table.get("z")?;
                        let new_obj = Object3d::new(&model_path, Vector3::new(x, y, z), rotation);
                        pending_objects_clone.borrow_mut().push(new_obj);
                        Ok(())
                    },
                )?,
            )?;
        }

        // Carga y ejecuta el script de Lua
        let update_script =
            std::fs::read_to_string("scripting/update.lua").expect("Could not read the Lua script");
        lua.load(&update_script).exec()?;

        Ok(Self {
            lua,
            objects,
            pending_objects,
        })
    }

    fn update_object_with_lua(
        lua: &Lua,
        obj: &mut Object3d,
        dt: f32,
        inputs: &Input,
    ) -> LuaResult<()> {
        let globals = lua.globals();
        let update_func: mlua::Function = globals.get("update")?;

        let obj_table = lua.create_table()?;
        {
            let pos_table = lua.create_table()?;
            pos_table.set("x", obj.position.x)?;
            pos_table.set("y", obj.position.y)?;
            pos_table.set("z", obj.position.z)?;
            obj_table.set("position", pos_table)?;
            obj_table.set("rotation", obj.rotation)?;
            obj_table.set("name", obj.object_name.clone())?;
            obj_table.set("id", obj.random_id)?;
            obj_table.set("render", obj.render)?;
        }

        let pressing_table = lua.create_table()?;
        for (i, key) in inputs.input.iter().enumerate() {
            pressing_table.set(i + 1, key.to_string())?;
        }

        let just_pressed_table = lua.create_table()?;
        for (i, key) in inputs.just_pressed.iter().enumerate() {
            just_pressed_table.set(i + 1, key.to_string())?;
        }

        let just_released_table = lua.create_table()?;
        for (i, key) in inputs.just_released.iter().enumerate() {
            just_released_table.set(i + 1, key.to_string())?;
        }

        let input_table = lua.create_table()?;
        input_table.set("pressing", pressing_table)?;
        input_table.set("just_pressed", just_pressed_table)?;

        let updated: Table = update_func.call((dt, obj_table, input_table))?;

        let pos_table: Table = updated.get("position")?;
        obj.position.x = pos_table.get("x")?;
        obj.position.y = pos_table.get("y")?;
        obj.position.z = pos_table.get("z")?;
        obj.rotation = updated.get("rotation")?;
        obj.render = updated.get("render")?;

        Ok(())
    }

    pub fn update(&self, dt: f32, inputs: &Input) -> LuaResult<()> {
        let lua = &self.lua;
        {
            let mut objects = self.objects.borrow_mut();
            for obj in objects.iter_mut() {
                Self::update_object_with_lua(lua, obj, dt, inputs)?;
            }
        }
        {
            let mut objects = self.objects.borrow_mut();
            let mut pending = self.pending_objects.borrow_mut();
            objects.append(&mut pending);
        }
        Ok(())
    }
}
