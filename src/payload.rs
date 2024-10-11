use std::{collections::HashMap, str::FromStr};

use avian3d::prelude::RigidBody;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, EnumString};
use yarnspinner::core::YarnValue;

#[derive(Serialize, Deserialize, Default, Clone, Deref, DerefMut, Debug)]
pub struct DirworldEntityPayload(Vec<DirworldComponent>);

impl DirworldEntityPayload {
    pub fn component(&self, name: &str) -> Option<&DirworldComponent> {
        if let Ok(discriminant) = DirworldComponentDiscriminants::from_str(name) {
            self.iter()
                .find(|component| discriminant == DirworldComponentDiscriminants::from(*component))
        } else {
            None
        }
    }

    pub fn component_mut(&mut self, name: &str) -> Option<&mut DirworldComponent> {
        if let Ok(discriminant) = DirworldComponentDiscriminants::from_str(name) {
            self.iter_mut()
                .find(|component| discriminant == DirworldComponentDiscriminants::from(&**component))
        } else {
            None
        }
    }

    pub fn components(&self, name: &str) -> Vec<&DirworldComponent> {
        if let Ok(discriminant) = DirworldComponentDiscriminants::from_str(name) {
            self.iter()
                .filter(|component| {
                    discriminant == DirworldComponentDiscriminants::from(*component)
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn components_mut(&mut self, name: &str) -> Vec<&mut DirworldComponent> {
        if let Ok(discriminant) = DirworldComponentDiscriminants::from_str(name) {
            self.iter_mut()
                .filter(|component| {
                    discriminant == DirworldComponentDiscriminants::from(&**component)
                })
                .collect()
        } else {
            vec![]
        }
    }
}

#[derive(Serialize, Deserialize, Clone, EnumDiscriminants, Debug)]
#[strum_discriminants(derive(EnumString))]
pub enum DirworldComponent {
    Transform(Transform),
    Name(String),
    Actor {
        local_variables: HashMap<String, YarnValue>,
        yarn_source: Vec<u8>,
    },
    Voice {
        pitch: i32,
        preset: i32,
        bank: i32,
        variance: u32,
        speed: f32,
    },
    Rigidbody(RigidBody),
    MeshCollider { 
        convex: bool,
        sensor: bool,
    },
    Script {
        lua_source: Vec<u8>,
    },
    Relationship {
        label: String,
        hash: [u8; 16],
    },
}
