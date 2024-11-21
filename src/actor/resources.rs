//! Actor-related resources

use bevy::prelude::*;
use yarnspinner::core::Library;

/// Library of yarnspinner function callbacks
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct FunctionLibrary(pub Library);
