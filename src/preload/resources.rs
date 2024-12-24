use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;

/// A map of asset handles required by each entry in a room, indexed by their paths
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct RoomAssets(pub HashMap<PathBuf, HashMap<String, UntypedHandle>>);
