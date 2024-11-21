use std::{collections::{BTreeMap, HashMap}, path::PathBuf};

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::Task};
use multi_key_map::MultiKeyMap;
use occule::Codec;
use uuid::Uuid;

use crate::payload::DirworldEntityPayload;

/// Root directory of the world
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DirworldRootDir(pub Option<PathBuf>);

/// Current directory of the world
#[derive(Resource, Default)]
pub struct DirworldCurrentDir{ 
    pub path: PathBuf,
    pub payload: Option<DirworldEntityPayload>,
}

/// Running background tasks
#[derive(Default, Resource, Deref, DerefMut)]
pub struct DirworldTasks(pub BTreeMap<String, Task<Option<CommandQueue>>>);

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct DirworldObservers(pub MultiKeyMap<EntryType, Entity>);

#[derive(Default, Resource, Deref, DerefMut)]
pub struct DirworldCodecs(pub MultiKeyMap<String, Box<dyn Codec + Send + Sync>>);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EntryType {
    File(Option<String>),
    Folder,
}

/// Structure containing payload data for cached (non-current) rooms
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct DirworldCache(pub HashMap<PathBuf, DirworldEntityPayload>);
