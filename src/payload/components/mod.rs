use std::collections::HashMap;

use avian3d::prelude::RigidBody;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use yarnspinner::core::YarnValue;

#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Transform(pub bevy::prelude::Transform);

#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Name(pub String);

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Actor {
    pub local_variables: HashMap<String, YarnValue>,
    pub yarn_source: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Voice {
    pub pitch: i32,
    pub preset: i32,
    pub bank: i32,
    pub variance: u32,
    pub speed: f32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            pitch: 60,
            preset: 0,
            bank: 0,
            variance: 3,
            speed: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Rigidbody(pub RigidBody);

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct MeshCollider {
    pub convex: bool,
    pub sensor: bool,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Script {
    pub lua_source: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Relationships(pub HashMap<String, [u8; 16]>);
