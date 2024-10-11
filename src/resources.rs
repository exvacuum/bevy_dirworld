use std::{collections::BTreeMap, path::PathBuf};

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::Task};
use multi_key_map::MultiKeyMap;
use occule::Codec;

/// Root directory of the world
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DirworldRootDir(pub Option<PathBuf>);

/// Current directory of the world
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DirworldCurrentDir(pub Option<PathBuf>);

/// Running background tasks
#[derive(Default, Resource, Deref, DerefMut)]
pub struct DirworldTasks(pub BTreeMap<String, Task<Option<CommandQueue>>>);

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub(crate) struct DirworldObservers(pub MultiKeyMap<EntryType, Entity>);

#[derive(Default, Resource, Deref, DerefMut)]
pub(crate) struct DirworldCodecs(pub MultiKeyMap<String, Box<dyn Codec + Send + Sync>>);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EntryType {
    File(Option<String>),
    Folder,
}
