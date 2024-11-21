use std::str::FromStr;

use bevy::prelude::*;
use bevy_scriptum::{
    runtimes::lua::{BevyEntity, BevyVec3, LuaRuntime, LuaScriptData},
    Runtime, ScriptingRuntimeBuilder,
};
use uuid::Uuid;

use crate::{components::DirworldEntity, conditionals::Condition};

pub fn trigger_update(
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
    scripting_runtime: Res<LuaRuntime>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (entity, mut script_data) in scripted_entities.iter_mut() {
        if let Err(e) = scripting_runtime.call_fn("on_update", &mut script_data, entity, (delta,)) {
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

pub fn register(
    runtime: ScriptingRuntimeBuilder<LuaRuntime>,
) -> ScriptingRuntimeBuilder<LuaRuntime> {
    register_fns!(
        runtime,
        translate,
        rotate,
        get_dirworld_id,
        condition_true,
        condition_ancestor_of,
        condition_descendant_of,
        condition_parent_of,
        condition_child_of,
        condition_in_room,
        condition_object_in_room
    )
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

fn get_dirworld_id(In((BevyEntity(entity),)): In<(BevyEntity,)>, dirworld_entity_query: Query<&DirworldEntity>) -> Option<String> {
    dirworld_entity_query.get(entity).ok().and_then(|entity| entity.payload.as_ref().map(|payload| payload.id.to_string()))
}

// Conditionals
fn condition_true(world: &mut World) -> bool {
    Condition::True.evaluate(world)
}

fn condition_ancestor_of(
    In((ancestor, descendant)): In<(String, String)>,
    world: &mut World,
) -> bool {
    let Ok(ancestor) = Uuid::from_str(&ancestor) else {
        warn!("Provided ancestor is not a valid UUID");
        return false;
    };
    let Ok(descendant) = Uuid::from_str(&descendant) else {
        warn!("Provided descendant is not a valid UUID");
        return false;
    };
    Condition::AncestorOf {
        ancestor,
        descendant,
    }
    .evaluate(world)
}

fn condition_descendant_of(
    In((descendant, ancestor)): In<(String, String)>,
    world: &mut World,
) -> bool {
    let Ok(ancestor) = Uuid::from_str(&ancestor) else {
        warn!("Provided ancestor is not a valid UUID");
        return false;
    };
    let Ok(descendant) = Uuid::from_str(&descendant) else {
        warn!("Provided descendant is not a valid UUID");
        return false;
    };
    Condition::DescendantOf {
        ancestor,
        descendant,
    }
    .evaluate(world)
}

fn condition_parent_of(In((parent, child)): In<(String, String)>, world: &mut World) -> bool {
    let Ok(parent) = Uuid::from_str(&parent) else {
        warn!("Provided parent is not a valid UUID");
        return false;
    };
    let Ok(child) = Uuid::from_str(&child) else {
        warn!("Provided child is not a valid UUID");
        return false;
    };
    Condition::ParentOf { parent, child }.evaluate(world)
}

fn condition_child_of(In((child, parent)): In<(String, String)>, world: &mut World) -> bool {
    let Ok(parent) = Uuid::from_str(&parent) else {
        warn!("Provided parent is not a valid UUID");
        return false;
    };
    let Ok(child) = Uuid::from_str(&child) else {
        warn!("Provided child is not a valid UUID");
        return false;
    };
    Condition::ChildOf { parent, child }.evaluate(world)
}

fn condition_in_room(In((room,)): In<(String,)>, world: &mut World) -> bool {
    let Ok(room) = Uuid::from_str(&room) else {
        warn!("Provided room is not a valid UUID");
        return false;
    };
    Condition::InRoom(room).evaluate(world)
}

fn condition_object_in_room(In((object,)): In<(String,)>, world: &mut World) -> bool {
    let Ok(object) = Uuid::from_str(&object) else {
        warn!("Provided object is not a valid UUID");
        return false;
    };
    Condition::ObjectInRoom(object).evaluate(world)
}

// }}}
