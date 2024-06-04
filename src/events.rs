use std::path::PathBuf;

use bevy::prelude::*;

/// Events related to activities in the dirworld.
#[derive(Event)]
pub enum DirworldNavigationEvent {
    /// Triggered when a room is left.
    LeftRoom {
        /// Path of room just left.
        path: PathBuf,
    },
    /// Triggered when a room is entered.
    EnteredRoom {
        /// Path of room just entered.
        path: PathBuf,
    },
}
