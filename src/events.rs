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

#[derive(Debug, Event, Deref, DerefMut, Clone)]
pub struct DirworldLeaveRoom(pub PathBuf);

#[derive(Debug, Event, Deref, DerefMut, Clone)]
pub struct DirworldEnterRoom(pub PathBuf);

#[derive(Debug, Event, Deref, DerefMut, Clone)]
pub struct DirworldChangeRoot(pub PathBuf);

#[derive(Event)]
pub struct DirworldSpawn {
    pub entity: Entity,
    pub data: Option<Vec<u8>>,
}
