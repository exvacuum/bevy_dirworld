use crate::cache::DirworldCache;
use crate::{
    components::DirworldEntity,
    resources::{DirworldCodecs, DirworldObservers, EntryType},
    utils::extract_entity_payload,
    Extensions,
};
use bevy::prelude::*;
use std::{collections::HashMap, path::PathBuf};

mod systems;

mod resources;
pub use resources::*;

mod events;
pub use events::DirworldPreload;

pub(crate) struct DirworldPreloadPlugin;

impl Plugin for DirworldPreloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            systems::handle_preload.run_if(in_state(PreloadState::Loading)),
        )
        .add_systems(OnEnter(PreloadState::Done), systems::handle_spawn)
        .init_resource::<RoomAssets>()
        .init_state::<PreloadState>();
    }
}

/// State of asset preloading
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum PreloadState {
    /// Indicates assets are in the process of loading
    #[default]
    Loading,
    /// Indicates all room assets are finished loading, i.e. all assets are loaded with
    /// dependencies
    Done,
}

/// Initiates loading of an asset
// TODO: Make into a command extension
pub fn load_entity(
    entry: &PathBuf,
    cache: &mut DirworldCache,
    codecs: &DirworldCodecs,
    observers: &DirworldObservers,
    commands: &mut Commands,
    preload_state: &mut NextState<PreloadState>,
    room_assets: &mut RoomAssets,
) {
    let (mut payload, data) = extract_entity_payload(&entry, &codecs);
    payload = payload.map(|p| cache.get_entity_cache(&entry).unwrap_or(p));
    let entry_type = if entry.is_dir() {
        EntryType::Folder
    } else {
        EntryType::File(entry.extensions())
    };
    let transform = payload
        .as_ref()
        .map(|payload| payload.transform.clone())
        .unwrap_or_default();
    let entity = commands
        .spawn((
            *transform,
            Visibility::Inherited,
            DirworldEntity {
                path: entry.clone(),
                payload,
            },
        ))
        .id();
    if let Some(observer) = observers.get(&entry_type) {
        preload_state.set(PreloadState::Loading);
        room_assets.insert(entry.clone(), HashMap::default());
        commands.trigger_targets(DirworldPreload { entity, data }, observer.clone());
        info!("Triggered preload for {entry:?}");
    }
}
