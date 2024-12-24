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

/// Event called when leaving a room
#[derive(Debug, Event, Deref, DerefMut, Clone)]
pub struct DirworldLeaveRoom(pub PathBuf);

/// Event called when entering a room
#[derive(Debug, Event, Deref, DerefMut, Clone)]
pub struct DirworldEnterRoom(pub PathBuf);

/// Event called when changing the world root
#[derive(Debug, Event, Deref, DerefMut, Clone)]
pub struct DirworldChangeRoot(pub PathBuf);

/// Event called to spawn a dirworld entities
#[derive(Event, Debug, Deref, DerefMut, Clone, Copy)]
pub struct DirworldSpawn(pub Entity);
