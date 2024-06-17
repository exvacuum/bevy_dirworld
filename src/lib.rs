#![warn(missing_docs)]

//! Plugin for bevy engine enabling interaction with and representation of the file system in the world.

use std::path::PathBuf;

use bevy::{asset::io::AssetSource, prelude::*};
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
    info!("building");
        let path_string = self.path.to_string_lossy().to_string();
        app.insert_resource(DirworldConfig::new(self.path.clone()))
            .register_asset_source("dirworld", AssetSource::build()
                    .with_reader(AssetSource::get_default_reader(path_string.clone()))
                    .with_watcher(|_| None))
            .add_event::<DirworldNavigationEvent>()
            .init_resource::<Dirworld>();
    }
}
