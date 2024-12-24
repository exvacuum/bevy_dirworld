use std::{str::FromStr, sync::Mutex};

use bevy::prelude::*;
use bevy_mod_scripting::api::providers::bevy_reflect::LuaVec3;
use bevy_mod_scripting::{api::providers::bevy_ecs::LuaEntity, lua::tealr::mlu::mlua::Error as LuaError};
use bevy_mod_scripting::lua::LuaEvent;
use bevy_mod_scripting::prelude::*;
use uuid::Uuid;

use crate::{components::DirworldEntity, conditionals::Condition};

pub fn trigger_update(mut w: PriorityEventWriter<LuaEvent<()>>) {
    let event = LuaEvent::<()> {
        args: (),
        hook_name: "on_update".into(),
        recipients: Recipients::All,
    };
    w.send(event, 0);
}

// ACTUAL API STUFF BELOW THIS POINT {{{

macro_rules! register_fns {
    ($runtime:expr, $($function:expr),+) => {
        {
            let ctx = $runtime.get_mut().unwrap();
            $(ctx.globals().set(stringify!($function).to_string(), ctx.create_function($function).unwrap()).unwrap();)+
        }
    };
}

pub fn register(api: &mut Mutex<Lua>) {
    register_fns!(
        api,
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

fn translate(ctx: &Lua, (entity, translation): (LuaEntity, LuaVec3)) -> Result<(), LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    if let Some(mut transform) = world.entity_mut(entity.inner().unwrap()).get_mut::<Transform>() {
        transform.translation += translation.inner().unwrap();
    }
    Ok(())
}

fn rotate(ctx: &Lua, (entity, axis, angle): (LuaEntity, LuaVec3, f32)) -> Result<(), LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    if let Some(mut transform) = world.entity_mut(entity.inner().unwrap()).get_mut::<Transform>() {
        transform.rotation *= Quat::from_axis_angle(axis.inner().unwrap(), angle);
    }
    Ok(())
}

fn get_dirworld_id(ctx: &Lua, (entity,): (LuaEntity,)) -> Result<String, LuaError> {
    let world = ctx.get_world()?;
    let world = world.read();
    if let Some(dirworld_entity) = world.entity(entity.inner().unwrap()).get::<DirworldEntity>() {
        dirworld_entity.payload.as_ref().map(|p| p.id.to_string()).ok_or(LuaError::runtime("Failed to get entity id from payload"))
    } else {
        Err(LuaError::runtime("Entity missing DirworldEntity component"))
    }
}

// Conditionals

pub struct ConditionalAPI;

impl APIProvider for ConditionalAPI {
    type APITarget = Mutex<Lua>;

    type ScriptContext = Mutex<Lua>;

    type DocTarget = LuaDocFragment;

    fn attach_api(
        &mut self,
        api: &mut Self::APITarget,
    ) -> Result<(), bevy_mod_scripting::prelude::ScriptError> {
        register(api);
        Ok(())
    }
}



fn condition_true(ctx: &Lua, _: ()) -> Result<bool, LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    Ok(Condition::True.evaluate(&mut world))
}

fn condition_ancestor_of(ctx: &Lua, (ancestor, descendant): (String, String)) -> Result<bool, LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    let Ok(ancestor) = Uuid::from_str(&ancestor) else {
        warn!("Provided ancestor is not a valid UUID");
        return Ok(false);
    };
    let Ok(descendant) = Uuid::from_str(&descendant) else {
        warn!("Provided descendant is not a valid UUID");
        return Ok(false);
    };
    Ok(Condition::AncestorOf {
        ancestor,
        descendant,
    }.evaluate(&mut world))
}

fn condition_descendant_of(ctx: &Lua, (descendant, ancestor): (String, String)) -> Result<bool, LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    let Ok(ancestor) = Uuid::from_str(&ancestor) else {
        warn!("Provided ancestor is not a valid UUID");
        return Ok(false);
    };
    let Ok(descendant) = Uuid::from_str(&descendant) else {
        warn!("Provided descendant is not a valid UUID");
        return Ok(false);
    };
    Ok(Condition::DescendantOf {
        ancestor,
        descendant,
    }.evaluate(&mut world))
}

fn condition_parent_of(ctx: &Lua, (parent, child): (String, String)) -> Result<bool, LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    let Ok(parent) = Uuid::from_str(&parent) else {
        warn!("Provided ancestor is not a valid UUID");
        return Ok(false);
    };
    let Ok(child) = Uuid::from_str(&child) else {
        warn!("Provided descendant is not a valid UUID");
        return Ok(false);
    };
    Ok(Condition::ParentOf {
        parent,
        child,
    }.evaluate(&mut world))
}

fn condition_child_of(ctx: &Lua, (child, parent): (String, String)) -> Result<bool, LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    let Ok(parent) = Uuid::from_str(&parent) else {
        warn!("Provided ancestor is not a valid UUID");
        return Ok(false);
    };
    let Ok(child) = Uuid::from_str(&child) else {
        warn!("Provided descendant is not a valid UUID");
        return Ok(false);
    };
    Ok(Condition::ChildOf {
        parent,
        child,
    }.evaluate(&mut world))
}

fn condition_in_room(ctx: &Lua, (room,): (String,)) -> Result<bool, LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    let Ok(room) = Uuid::from_str(&room) else {
        warn!("Provided room is not a valid UUID");
        return Ok(false);
    };
    Ok(Condition::InRoom(room).evaluate(&mut world))
}

fn condition_object_in_room(ctx: &Lua, (object,): (String,)) -> Result<bool, LuaError> {
    let world = ctx.get_world()?;
    let mut world = world.write();
    let Ok(object) = Uuid::from_str(&object) else {
        warn!("Provided object is not a valid UUID");
        return Ok(false);
    };
    Ok(Condition::ObjectInRoom(object).evaluate(&mut world))
}

// }}}
