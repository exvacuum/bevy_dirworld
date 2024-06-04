use bevy::prelude::*;

/// A tooltip on an object, which can be displayed.
#[derive(Component)]
pub struct Tooltip(pub String);
