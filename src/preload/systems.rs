use bevy::prelude::*;

use crate::{components::DirworldEntity, events::DirworldSpawn, resources::{DirworldObservers, EntryType}, Extensions};

use super::{PreloadState, RoomAssets};

pub fn handle_preload(
    asset_server: Res<AssetServer>,
    room_assets: Res<RoomAssets>,
    mut next_state: ResMut<NextState<PreloadState>>,
) {
    if room_assets.is_empty()
        || room_assets
            .values()
            .flat_map(|v| v.values())
            .all(|a| asset_server.is_loaded_with_dependencies(a))
    {
        info!("Preload Done.");
        next_state.set(PreloadState::Done);
    }
}

pub fn handle_spawn(
    dirworld_entity_query: Query<(Entity, &DirworldEntity)>,
    mut commands: Commands,
    observers: Res<DirworldObservers>,
) {
    info!("Spawning");
    for (entity, DirworldEntity { path, .. }) in dirworld_entity_query.iter() {
        let entry_type = if path.is_dir() {
            EntryType::Folder
        } else {
            EntryType::File(path.extensions())
        };
        if let Some(observer) = observers.get(&entry_type) {
            info!("Found observer {observer:?} for {entry_type:?}");
            commands.trigger_targets(DirworldSpawn(entity), observer.clone());
        }
    }
}

