use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod components;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct DirworldEntityPayload { 
    pub id: Uuid,
    pub transform: components::Transform,
    pub name: Option<components::Name>,
    pub actor: Option<components::Actor>,
    pub voice: Option<components::Voice>,
    pub rigidbody: Option<components::Rigidbody>,
    pub mesh_collider: Option<components::MeshCollider>,
    pub scripts: Option<Vec<components::Script>>,
    pub relationships: Option<components::Relationships>,
}

impl DirworldEntityPayload {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }
}

