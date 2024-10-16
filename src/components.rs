use std::path::PathBuf;

use bevy::prelude::*;

use crate::payload::DirworldEntityPayload;

/// A tooltip on an object, which can be displayed.
#[derive(Component)]
pub struct Tooltip(pub String);

/// A marker component for entities spawned by dirworld handlers, i.e. they should be removed when the room changes.
#[derive(Component, Clone, Debug)]
pub struct DirworldEntity { 
    pub path: PathBuf,
    pub payload: Option<DirworldEntityPayload>,
}

#[derive(Debug, Component)]
pub struct Persist;
