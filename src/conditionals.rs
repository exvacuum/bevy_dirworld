use bevy::{
    ecs::system::SystemState,
    prelude::{AncestorIter, Entity, Parent, Query, World},
};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use uuid::Uuid;

use crate::{components::DirworldEntity, resources::DirworldCurrentDir};

// I Store Conditions as Enum Data
#[derive(Serialize, Deserialize, AsRefStr, Debug, Default, Clone, PartialEq, Eq)]
pub enum Condition {
    #[default]
    #[strum(serialize = "Always True")]
    True,
    #[strum(serialize = "Child Of")]
    ChildOf {
        child: Uuid,
        parent: Uuid,
    },
    #[strum(serialize = "Parent Of")]
    ParentOf {
        parent: Uuid,
        child: Uuid,
    },
    #[strum(serialize = "Descendant Of")]
    DescendantOf {
        descendant: Uuid,
        ancestor: Uuid,
    },
    #[strum(serialize = "Ancestor Of")]
    AncestorOf {
        ancestor: Uuid,
        descendant: Uuid,
    },
    #[strum(serialize = "In Room")]
    InRoom(Uuid),
    #[strum(serialize = "Object In Room")]
    ObjectInRoom(Uuid),
}

impl Condition {
    pub fn evaluate(&self, world: &mut World) -> bool {
        match self {
            Condition::True => true,
            Condition::ChildOf { child, parent } => parent_of(world, *parent, *child),
            Condition::ParentOf { parent, child } => parent_of(world, *parent, *child),
            Condition::DescendantOf {
                descendant,
                ancestor,
            } => ancestor_of(world, *ancestor, *descendant),
            Condition::AncestorOf {
                ancestor,
                descendant,
            } => ancestor_of(world, *ancestor, *descendant),
            Condition::InRoom(uuid) => in_room(world, *uuid),
            Condition::ObjectInRoom(uuid) => object_in_room(world, *uuid),
        }
    }

    pub fn get_api_function_name(&self) -> &'static str {
        match self {
            Condition::True => "conditional_true",
            Condition::ChildOf { .. } => "conditional_child_of",
            Condition::ParentOf { .. } => "conditional_parent_of",
            Condition::DescendantOf { .. } => "conditional_descendant_of",
            Condition::AncestorOf { .. } =>"conditional_ancestor_of",
            Condition::InRoom(_) => "conditional_in_room",
            Condition::ObjectInRoom(_) => "conditional_object_in_room",
        }
    }

    pub fn from_api_function_name_and_args(name: &str, args: &[&str]) -> Option<Self> {
        match name {
            "conditional_true" => Some(Condition::True),
            "conditional_child_of" => {
                let Some(child) = args.get(0).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                let Some(parent) = args.get(1).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                Some(Condition::ChildOf { child, parent })
            }
            "conditional_parent_of" => {
                let Some(child) = args.get(1).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                let Some(parent) = args.get(0).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                Some(Condition::ParentOf { child, parent })
            }
            "conditional_descendant_of" => {
                let Some(descendant) = args.get(0).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                let Some(ancestor) = args.get(1).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                Some(Condition::DescendantOf { descendant, ancestor })
            }
            "conditional_ancestor_of" => {
                let Some(descendant) = args.get(1).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                let Some(ancestor) = args.get(0).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                Some(Condition::AncestorOf { descendant, ancestor })
            }
            "condtitional_in_room" => {
                let Some(room_id) = args.get(0).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                Some(Condition::InRoom(room_id))
            }
            "condtitional_object_in_room" => {
                let Some(object_id) = args.get(0).and_then(|arg| Uuid::parse_str(arg).ok()) else {
                    return None;
                };
                Some(Condition::ObjectInRoom(object_id))
            }
            _ => None,
        }
    }
}

// Condition Checkers beyond this point

fn ancestor_of(world: &mut World, ancestor: Uuid, descendant: Uuid) -> bool {
    let mut system_state =
        SystemState::<(Query<(Entity, &DirworldEntity)>, Query<&Parent>)>::new(world);
    let (dirworld_entities, parents) = system_state.get(world);
    let Some((ancestor_entity, _)) = dirworld_entities.iter().find(|(_, entity)| {
        entity
            .payload
            .as_ref()
            .is_some_and(|payload| payload.id == ancestor)
    }) else {
        return false;
    };

    let Some((descendant_entity, _)) = dirworld_entities.iter().find(|(_, entity)| {
        entity
            .payload
            .as_ref()
            .is_some_and(|payload| payload.id == descendant)
    }) else {
        return false;
    };

    AncestorIter::new(&parents, descendant_entity)
        .find(|descendant| *descendant == ancestor_entity)
        .is_some()
}

fn parent_of(world: &mut World, parent: Uuid, child: Uuid) -> bool {
    let mut system_state =
        SystemState::<(Query<(Entity, &DirworldEntity)>, Query<&Parent>)>::new(world);
    let (dirworld_entities, parents) = system_state.get(world);
    let Some((parent_entity, _)) = dirworld_entities.iter().find(|(_, entity)| {
        entity
            .payload
            .as_ref()
            .is_some_and(|payload| payload.id == parent)
    }) else {
        return false;
    };

    let Some((child_entity, _)) = dirworld_entities.iter().find(|(_, entity)| {
        entity
            .payload
            .as_ref()
            .is_some_and(|payload| payload.id == child)
    }) else {
        return false;
    };

    parents
        .get(child_entity)
        .is_ok_and(|parent| parent.get() == parent_entity)
}

fn in_room(world: &mut World, room: Uuid) -> bool {
    let current_dir = world.resource::<DirworldCurrentDir>();
    current_dir.payload.as_ref().is_some_and(|payload| payload.id == room)
}

fn object_in_room(world: &mut World, object: Uuid) -> bool {
    let mut dirworld_entities = world.query::<&DirworldEntity>();
    dirworld_entities
        .iter(world)
        .find(|entity| {
            entity
                .payload
                .as_ref()
                .is_some_and(|payload| payload.id == object)
        })
        .is_some()
}
