use std::collections::HashMap;

use avian3d::prelude::RigidBody;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use yarnspinner::core::YarnValue;

/// Payload component that corresponds to [`bevy::prelude::Transform`]
#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Transform(pub bevy::prelude::Transform);

/// Payload component that represent's an entity's name
#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Name(pub String);

/// Payload component that represents a yarnspinner actor
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Actor {
    /// Actor-local variables
    pub local_variables: HashMap<String, YarnValue>,
    /// Source for the yarnspinner dialog
    pub yarn_source: Vec<u8>,
}

/// Payload component that represents a character's voice. Uses rustysynth to generate random MIDI
/// tones based on given parameters.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Voice {
    /// Base MIDI pitch of voice. Defaults to 60
    pub pitch: i32,
    /// MIDI preset to use for voice. Defaults to 0
    pub preset: i32,
    /// MIDI bank to use. Defaults to 0
    pub bank: i32,
    /// Variance in pitch of voice. Defaults to 3
    pub variance: u32,
    /// Speed of voice. Defaults to 1.0
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

/// Payload component that wraps a [`avian3d::prelude::RigidBody`]
#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Rigidbody(pub RigidBody);

/// Payload component that represents mesh colliders that will be generated for this entity
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct MeshCollider {
    /// Whether the generated colliders should be convex hulls
    pub convex: bool,
    /// Whether the generated colliders should be triggers
    pub sensor: bool,
}

/// Payload component representing a lua script that will be attached to an entity
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Script {
    /// Lua script source
    pub lua_source: Vec<u8>,
}

/// Payload component for an arbitrary relationship map, can store 128-bit identifiers indexed by names
#[derive(Serialize, Deserialize, Clone, Default, Deref, DerefMut, Debug)]
pub struct Relationships(pub HashMap<String, [u8; 16]>);

/// Payload component that indicates that this entity should be able to be picked up
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Pickup;
