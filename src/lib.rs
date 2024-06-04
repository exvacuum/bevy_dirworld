#![warn(missing_docs)]

//! Plugin for bevy engine enabling interaction with and representation of the file system in the world.

use std::path::PathBuf;

use bevy::prelude::*;
use events::DirworldNavigationEvent;
use resources::{Dirworld, DirworldConfig};

/// Components used by this plugin
pub mod components;

/// Events used by this plugin
pub mod events;

/// Resources used by this plugin
pub mod resources;

/// Plugin which enables high-level interaction
pub struct DirworldPlugin {
    /// Root path of world
    pub path: PathBuf,
}

impl Plugin for DirworldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DirworldConfig::new(self.path.clone()))
            .add_event::<DirworldNavigationEvent>()
            .init_resource::<Dirworld>();
    }
}
