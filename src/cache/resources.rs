use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;

use crate::{components::DirworldEntity, payload::DirworldEntityPayload};

/// Structure containing payload data for cached (non-current) rooms
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct DirworldCache(pub HashMap<PathBuf, DirworldEntityPayload>);

impl DirworldCache {
    /// Stores an entity's payload in the cache, if it exists
    pub fn cache_entity(&mut self, dirworld_entity: &DirworldEntity) {
        if let Some(payload) = &dirworld_entity.payload {
            self.insert(dirworld_entity.path.clone(), payload.clone());
        }
    }
    
    pub fn get_entity_cache(&mut self, path: impl Into<PathBuf>) -> Option<DirworldEntityPayload> {
        self.remove(&path.into())
    }
}
