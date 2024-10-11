use bevy::{prelude::*, utils::HashMap};
use yarnspinner::{core::LineId, runtime::Dialogue};

#[derive(Component)]
pub struct Actor {
    pub dialogue: Dialogue,
    pub metadata: HashMap<LineId, Vec<String>>,
}
