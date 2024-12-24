use bevy::prelude::*;

/// Event used to trigger preload callbacks after the asset file has been pre-processed to extract
/// the payload
#[derive(Debug, Event, Clone)]
pub struct DirworldPreload {
    /// Entity with the `[DirworldEntity]` component corresponding to the entity being preloaded
    pub entity: Entity,
    /// The data portion of the file after being pre-processed
    pub data: Option<Vec<u8>>,
}

