//! NPCs containing their own individual yarnspinner contexts
// TODO: Split off into own crate?

use std::sync::{Arc, Mutex};

use bevy::{prelude::*, utils::HashMap};
use lazy_static::lazy_static;
use resources::FunctionLibrary;
use yarnspinner::{core::{LineId, YarnValue}, runtime::Dialogue};

pub mod components;
pub mod events;
pub mod resources;
mod systems;

lazy_static! {
    /// Custom yarnspinner variable storage
    /// Stores variables as <instance>.<varname>
    /// Global variables are stored in the "global" instance
    pub static ref DIRWORLD_VARIABLE_STORAGE: Arc<Mutex<DirworldVariableStorage>> =
        Arc::new(Mutex::new(DirworldVariableStorage::default()));
}

/// Plugin which controls the behavior of actors
pub struct ActorPlugin {
    pub custom_function_registration: Option<fn(&mut FunctionLibrary)>,
}

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        let mut function_library = FunctionLibrary::default();
        function_library.add_function("get_string", get_string);
        function_library.add_function("get_number", get_number);
        function_library.add_function("get_bool", get_bool);
        if let Some(custom_function_registration) = &self.custom_function_registration {
            (custom_function_registration)(&mut function_library)
        }

        app.add_systems(
            Update,
            (systems::handle_dialog_initiation, systems::progress_dialog, systems::handle_variable_set_commands),
        )
        .insert_resource(function_library)
        .add_event::<events::ContinueDialogueEvent>()
        .add_event::<events::DialogueEvent>();
    }
}

fn get_string(instance_name: &str, var_name: &str) -> String {
    if let Some(YarnValue::String(value)) = DIRWORLD_VARIABLE_STORAGE
        .lock()
        .unwrap()
        .get(instance_name, var_name)
    {
        value
    } else {
        "".into()
    }
}

fn get_number(instance_name: &str, var_name: &str) -> f32 {
    if let Some(YarnValue::Number(value)) = DIRWORLD_VARIABLE_STORAGE
        .lock()
        .unwrap()
        .get(instance_name, var_name)
    {
        value
    } else {
        0.0
    }
}

fn get_bool(instance_name: &str, var_name: &str) -> bool {
    if let Some(YarnValue::Boolean(value)) = DIRWORLD_VARIABLE_STORAGE
        .lock()
        .unwrap()
        .get(instance_name, var_name)
    {
        value
    } else {
        false
    }
}

/// Variable Storage
#[derive(Default, Debug)]
pub struct DirworldVariableStorage(pub HashMap<String, YarnValue>);

impl DirworldVariableStorage {
    /// Set value of instance variable (use "global" for global)
    pub fn set(&mut self, instance_name: &str, var_name: &str, value: YarnValue) {
        self.0.insert(format!("{instance_name}.{var_name}"), value);
    }

    /// Get value of instance variable (use "global" for global)
    pub fn get(&self, instance_name: &str, var_name: &str) -> Option<YarnValue> {
        self.0.get(&format!("{instance_name}.{var_name}")).cloned()
    }
}
