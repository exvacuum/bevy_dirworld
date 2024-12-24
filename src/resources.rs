use std::{collections::BTreeMap, path::PathBuf};

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::Task};
use multi_key_map::MultiKeyMap;
use occule::Codec;

use crate::payload::DirworldEntityPayload;

/// Root directory of the world
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DirworldRootDir(pub Option<PathBuf>);

/// Current directory of the world
#[derive(Resource, Default)]
pub struct DirworldCurrentDir{ 
    /// Path of current directory
    pub path: PathBuf,
    /// Payload (contents of .door file) in current directory, if present
    pub payload: Option<DirworldEntityPayload>,
}

/// Running background tasks
#[derive(Default, Resource, Deref, DerefMut)]
pub struct DirworldTasks(pub BTreeMap<String, Task<Option<CommandQueue>>>);

/// A map between file types and their corresponding preload/spawn callback observers
#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct DirworldObservers(pub MultiKeyMap<EntryType, Entity>);

/// A map between file extensions and their corresponding [`Codec`]s
#[derive(Default, Resource, Deref, DerefMut)]
pub struct DirworldCodecs(pub MultiKeyMap<String, Box<dyn Codec + Send + Sync>>);

/// Type of a filesystem entry
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EntryType {
    /// A file with an optional extension
    File(Option<String>),
    /// A folder
    Folder,
}

