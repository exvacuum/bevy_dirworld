use bevy::prelude::*;
use bevy_scriptum::{runtimes::lua::{BevyEntity, BevyVec3, LuaRuntime, LuaScriptData}, ScriptingRuntimeBuilder, Runtime};

pub fn trigger_update(
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
    scripting_runtime: Res<LuaRuntime>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (entity, mut script_data) in scripted_entities.iter_mut() {
        if let Err(e) = scripting_runtime.call_fn("on_update", &mut script_data, entity, (delta, )) {
            error!("Encountered lua scripting error: {:?}", e);
        }
    }
}

// ACTUAL API STUFF BELOW THIS POINT {{{

macro_rules! register_fns {
    ($runtime:expr, $($function:expr),+) => {
        { 
            $runtime$(.add_function(stringify!($function).to_string(), $function))+
        }
    };
}

pub fn register(runtime: ScriptingRuntimeBuilder<LuaRuntime>) -> ScriptingRuntimeBuilder<LuaRuntime> {
    register_fns!(runtime, translate, rotate)
}

fn translate(
    In((BevyEntity(entity), BevyVec3(translation))): In<(BevyEntity, BevyVec3)>,
    mut transform_query: Query<&mut Transform>,
) {
    if let Ok(mut transform) = transform_query.get_mut(entity) {
        transform.translation += translation;
    }
}

fn rotate(
    In((BevyEntity(entity), BevyVec3(axis), angle)): In<(BevyEntity, BevyVec3, f32)>,
    mut transform_query: Query<&mut Transform>,
) {
    if let Ok(mut transform) = transform_query.get_mut(entity) {
        if let Ok(direction) = Dir3::new(axis) {
            transform.rotate_axis(direction, angle);
        } else {
            warn!("Provided axis was not a valid direction!");
        }
    }
}

// }}} 
