use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use bevy::prelude::*;

use crate::events::DirworldNavigationEvent;

/// Configuration for Dirworld.
#[derive(Resource)]
pub struct DirworldConfig {
    root: PathBuf,
}

impl DirworldConfig {
    /// Construct a new dirworld config with the given root path. Will panic if the provided path
    /// cannot be canonicalized.
    // TODO: Don't panic? lol
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: fs::canonicalize(root).expect("Failed to canonicalize path!"),
        }
    }

    ///
    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}

/// Contains the dirworld state.
#[derive(Resource)]
pub struct Dirworld {
    /// Current active directory.
    pub path: PathBuf,

    /// Entities local to the current room.
    pub tracked_entities: Vec<Entity>,
}

impl FromWorld for Dirworld {
    fn from_world(world: &mut World) -> Self {
        let config = world.remove_resource::<DirworldConfig>().unwrap();
        world.send_event(DirworldNavigationEvent::EnteredRoom {
            path: config.root().clone(),
        });
        let result = Self {
            path: config.root().clone(),
            tracked_entities: vec![],
        };
        world.insert_resource(config);
        result
    }
}

impl Dirworld {
    /// Move into a new room.
    // TODO: Clear tracked entities?
    // TODO: Make into command extension trait?
    pub fn navigate_to(
        &mut self,
        path: PathBuf,
        event_writer: &mut EventWriter<DirworldNavigationEvent>,
    ) -> Result<()> {
        event_writer.send(DirworldNavigationEvent::LeftRoom {
            path: self.path.clone(),
        });
        self.path = Path::new(&self.path)
            .join(path)
            .to_str()
            .context("Path not valid UTF-8")?
            .into();
        event_writer.send(DirworldNavigationEvent::EnteredRoom {
            path: self.path.clone(),
        });
        Ok(())
    }
}
