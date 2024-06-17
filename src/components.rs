use std::path::PathBuf;

use bevy::prelude::*;

/// A tooltip on an object, which can be displayed.
#[derive(Component)]
pub struct Tooltip(pub String);

/// A marker component for entities spawned by dirworld handlers, i.e. they should be removed when the room changes.
#[derive(Component)]
pub struct DirworldEntity(pub PathBuf);
