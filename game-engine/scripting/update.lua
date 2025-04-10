local function update_cube(dt, object, inputs)
    SPEED = 15.0
    object.position.z = object.position.z + (SPEED * dt)

    if object.position.z > 40.0 then
        object.render = false
    end

    return object
end

local function update_aircraft(dt, object, inputs, utils)
    SPEED = 10.0

    if utils.has_value(inputs.pressing, 'A') then
        object.position.x = object.position.x + (SPEED * dt)
    end
    if utils.has_value(inputs.pressing, 'D') then
        object.position.x = object.position.x - (SPEED * dt)
    end
    if utils.has_value(inputs.pressing, 'W') then
        object.position.y = object.position.y - (SPEED * dt)
    end
    if utils.has_value(inputs.pressing, 'S') then
        object.position.y = object.position.y + (SPEED * dt)
    end

    if utils.has_value(inputs.just_pressed, 'P') then
        play_sound('audio/blaster.wav')
        new_model('model/cube.obj', {
                x = object.position.x,
                y = object.position.y,
                z = object.position.z
            },
            object.rotation)
    end

    return object
end


function update(dt, object, inputs)
    local utils = require('scripting/utils')

    if object.name == 'model/cube.obj' then
        return update_cube(dt, object, inputs)
    end

    if object.name == 'model/aircraft.obj' then
        return update_aircraft(dt, object, inputs, utils)
    end

    return object
end
