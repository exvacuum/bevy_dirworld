// #![warn(missing_docs)]

//! Plugin for bevy engine enabling interaction with and representation of the file system in the world.

use std::{ffi::OsStr, path::PathBuf};

use actor::ActorPlugin;
use bevy::render::mesh::ExtrusionBuilder;
use bevy::{ecs::system::IntoObserverSystem, prelude::*};
use bevy_scriptum::{runtimes::lua::LuaRuntime, BuildScriptingRuntime, ScriptingRuntimeBuilder};
use events::{
    DirworldChangeRoot, DirworldEnterRoom, DirworldLeaveRoom, DirworldNavigationEvent,
    DirworldSpawn,
};
use occule::Codec;
use resources::DirworldCache;
use resources::{
    DirworldCodecs, DirworldCurrentDir, DirworldObservers, DirworldRootDir, DirworldTasks,
    EntryType,
};
pub use watcher::DirworldWatcherEvent;
pub use watcher::DirworldWatcherSet;
use yarnspinner::core::Library;

/// Components used by this plugin
pub mod components;

/// Events used by this plugin
pub mod events;

/// Resources used by this plugin
pub mod resources;

mod watcher;

/// Commands for this plugin
pub mod commands;

mod systems;

mod observers;

pub mod utils;

/// Payload for dirworld entities
pub mod payload;

/// Actor component
pub mod actor;

mod lua_api;

pub mod conditionals;

pub mod yarnspinner_api;

pub mod room_generation;

/// Plugin which enables high-level interaction
#[derive(Default)]
pub struct DirworldPlugin {
    pub register_custom_lua_api:
        Option<Box<dyn Fn(ScriptingRuntimeBuilder<LuaRuntime>) + Send + Sync>>,
}

impl Plugin for DirworldPlugin {
    fn build(&self, app: &mut App) {
        info!("building");
        app.add_plugins(ActorPlugin {
            custom_function_registration: Some(yarnspinner_api::setup_yarnspinner_functions),
        })
        .add_systems(Startup, watcher::setup)
        .add_systems(
            Update,
            (
                systems::remove_completed_tasks,
                lua_api::trigger_update,
                yarnspinner_api::process_commands,
            ),
        )
        .add_systems(
            PostUpdate,
            watcher::update,
        )
        .add_scripting::<LuaRuntime>(|runtime| {
            let runtime = lua_api::register(runtime);
            if let Some(register_custom) = &self.register_custom_lua_api {
                (register_custom)(runtime);
            }
        })
        .init_resource::<DirworldCache>()
        .init_resource::<DirworldRootDir>()
        .init_resource::<DirworldCurrentDir>()
        .init_resource::<DirworldTasks>()
        .init_resource::<DirworldObservers>()
        .init_resource::<DirworldCodecs>()
        .add_event::<DirworldEnterRoom>()
        .add_event::<DirworldLeaveRoom>()
        .add_event::<DirworldChangeRoot>()
        .add_event::<DirworldWatcherEvent>()
        .observe(observers::navigate_to_room)
        .observe(observers::handle_changes)
        .observe(observers::change_root)
        .observe(observers::navigate_from_room);
    }
}

pub trait Extensions {
    fn extensions(&self) -> Option<String>;

    fn file_stem_no_extensions(&self) -> Option<String>;

    fn no_extensions(&self) -> PathBuf;
}

impl Extensions for PathBuf {
    fn extensions(&self) -> Option<String> {
        let mut temp_path = self.clone();
        let mut extensions = Vec::<String>::new();
        while let Some(extension) = temp_path.extension() {
            extensions.insert(0, extension.to_string_lossy().into());
            temp_path.set_extension("");
        }
        if extensions.is_empty() {
            None
        } else {
            Some(extensions.join("."))
        }
    }

    fn file_stem_no_extensions(&self) -> Option<String> {
        let mut temp_path = self.clone();
        while let Some(_) = temp_path.extension() {
            temp_path.set_extension("");
        }
        temp_path
            .file_stem()
            .and_then(OsStr::to_str)
            .map(str::to_string)
    }

    fn no_extensions(&self) -> PathBuf {
        let mut temp_path = self.clone();
        while let Some(_) = temp_path.extension() {
            temp_path.set_extension("");
        }
        temp_path
    }
}

pub trait DirworldApp {
    fn register_dirworld_entry_callback<B: Bundle, M>(
        &mut self,
        extensions: Vec<EntryType>,
        observer: impl IntoObserverSystem<DirworldSpawn, B, M>,
    ) -> &mut Self;

    fn register_dirworld_entry_codec<C: Codec + Send + Sync + 'static>(
        &mut self,
        extensions: Vec<String>,
        codec: C,
    ) -> &mut Self;
}

impl DirworldApp for App {
    fn register_dirworld_entry_callback<B: Bundle, M>(
        &mut self,
        extensions: Vec<EntryType>,
        observer: impl IntoObserverSystem<DirworldSpawn, B, M>,
    ) -> &mut Self {
        let world = self.world_mut();
        let observer_entity_id;

        {
            let mut observer_entity = world.spawn_empty();
            observer_entity_id = observer_entity.id();
            observer_entity.insert(Observer::new(observer).with_entity(observer_entity_id));
        }

        world.flush();
        world
            .resource_mut::<DirworldObservers>()
            .insert_many(extensions, observer_entity_id);
        self
    }

    fn register_dirworld_entry_codec<C: Codec + Send + Sync + 'static>(
        &mut self,
        extensions: Vec<String>,
        codec: C,
    ) -> &mut Self {
        self.world_mut()
            .resource_mut::<DirworldCodecs>()
            .insert_many(extensions, Box::new(codec));
        self
    }
}
