use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payload components
pub mod components;

/// Payload steganographically embedded into asset files
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct DirworldEntityPayload { 
    /// Unique identifier for this entity, used by conditional system
    pub id: Uuid,
    /// Transform of this entity
    pub transform: components::Transform,
    /// Name for this entity
    pub name: Option<components::Name>,
    /// Actor information for this entity
    pub actor: Option<components::Actor>,
    /// Voice information for this entity
    pub voice: Option<components::Voice>,
    /// Rigidbody for this entity
    pub rigidbody: Option<components::Rigidbody>,
    /// Mesh collider information for this entity
    pub mesh_collider: Option<components::MeshCollider>,
    /// Lua scripts for this entity
    pub scripts: Option<Vec<components::Script>>,
    /// Relationships for this entity
    pub relationships: Option<components::Relationships>,
    /// Pickup information for this entity
    pub pickup: Option<components::Pickup>,
}

impl DirworldEntityPayload {
    /// Create a new default payload with a randomized UUID
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }
}

