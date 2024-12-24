#![warn(missing_docs)]

//! Plugin for bevy engine enabling interaction with and representation of the file system in the world.

use std::{ffi::OsStr, path::PathBuf};

use actor::ActorPlugin;
use bevy::{ecs::system::IntoObserverSystem, prelude::*};
use bevy_mod_scripting::core::{AddScriptApiProvider, AddScriptHost, AddScriptHostHandler, ScriptingPlugin};
use bevy_mod_scripting::lua::LuaScriptHost;
use cache::DirworldCache;
use events::{DirworldChangeRoot, DirworldEnterRoom, DirworldLeaveRoom, DirworldSpawn};
use occule::Codec;
use preload::{DirworldPreload, DirworldPreloadPlugin};
use resources::EntryType;
use resources::{
    DirworldCodecs, DirworldCurrentDir, DirworldObservers, DirworldRootDir, DirworldTasks,
};
pub use watcher::DirworldWatcherEvent;
pub use watcher::DirworldWatcherSet;

/// Components used by this plugin
pub mod components;

/// Events used by this plugin
pub mod events;

/// Resources used by this plugin
pub mod resources;

/// Commands for this plugin
pub mod commands;

/// Utility functions
pub mod utils;

/// Payload for dirworld entities
pub mod payload;

/// Actor component
pub mod actor;

/// System for dirworld-related condition checking
pub mod conditionals;

/// Room/asset preloading
pub mod preload;

mod cache;

mod yarnspinner_api;

mod lua_api;

mod systems;

mod observers;

mod watcher;

/// Plugin which enables high-level interaction
#[derive(Default)]
pub struct DirworldPlugin;

impl Plugin for DirworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ActorPlugin {
                custom_function_registration: Some(yarnspinner_api::setup_yarnspinner_functions),
            },
            DirworldPreloadPlugin,
            ScriptingPlugin,
        ))
        .add_systems(Startup, watcher::setup)
        .add_systems(
            Update,
            (
                systems::remove_completed_tasks,
                lua_api::trigger_update,
                yarnspinner_api::process_commands,
            ),
        )
        .add_script_host::<LuaScriptHost<()>>(PostUpdate)
        .add_script_handler::<LuaScriptHost<()>, 0, 0>(PostUpdate)
        .add_api_provider::<LuaScriptHost<()>>(Box::new(lua_api::ConditionalAPI))
        .add_systems(PostUpdate, watcher::update)
        .init_resource::<DirworldRootDir>()
        .init_resource::<DirworldCache>()
        .init_resource::<DirworldCurrentDir>()
        .init_resource::<DirworldTasks>()
        .init_resource::<DirworldObservers>()
        .init_resource::<DirworldCodecs>()
        .add_event::<DirworldEnterRoom>()
        .add_event::<DirworldLeaveRoom>()
        .add_event::<DirworldChangeRoot>()
        .add_event::<DirworldWatcherEvent>()
        .add_observer(observers::navigate_to_room)
        .add_observer(observers::handle_changes)
        .add_observer(observers::change_root)
        .add_observer(observers::navigate_from_room);
    }
}

/// Extension trait for working with multiple file extensions on paths
pub trait Extensions {
    /// Get all the extensions on this path if applicable
    fn extensions(&self) -> Option<String>;

    /// Gets the file stem (without any extensions) of this path if applicable
    fn file_stem_no_extensions(&self) -> Option<String>;

    /// Gets the path with any extensions removed
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

/// Extension trait providing functions for registering callbacks and codecs for filesystem entries
pub trait DirworldApp {
    /// Register callbacks to be executed when a file with given [`EntryType`]s is loaded. The
    /// `preload_callback` parameter controls loading assets and is called before spawning any
    /// entities in the room, and the `spawn_callback` handles initializing the spawned entities.
    fn register_dirworld_entry_callbacks<B: Bundle, M, PB: Bundle, PM>(
        &mut self,
        extensions: Vec<EntryType>,
        preload_callback: Option<impl IntoObserverSystem<DirworldPreload, PB, PM>>,
        spawn_callback: impl IntoObserverSystem<DirworldSpawn, B, M>,
    ) -> &mut Self;

    /// Register a [`Codec`] to be used to extract [`crate::payload::DirworldEntityPayload`]s from
    /// files with matching extensions.
    fn register_dirworld_entry_codec<C: Codec + Send + Sync + 'static>(
        &mut self,
        extensions: Vec<String>,
        codec: C,
    ) -> &mut Self;
}

impl DirworldApp for App {
    fn register_dirworld_entry_callbacks<B: Bundle, M, PB: Bundle, PM>(
        &mut self,
        extensions: Vec<EntryType>,
        preload_callback: Option<impl IntoObserverSystem<DirworldPreload, PB, PM>>,
        spawn_observer: impl IntoObserverSystem<DirworldSpawn, B, M>,
    ) -> &mut Self {
        let world = self.world_mut();
        let observer_entity_id;

        {
            let mut observer_entity = world.spawn_empty();
            observer_entity_id = observer_entity.id();
            if let Some(preload_callback) = preload_callback {
                observer_entity.with_children(|parent| {
                    parent.spawn(Observer::new(preload_callback).with_entity(observer_entity_id));
                });
            }
            observer_entity.with_children(|parent| {
                parent.spawn(Observer::new(spawn_observer).with_entity(observer_entity_id));
            });
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
