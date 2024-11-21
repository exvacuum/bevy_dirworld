use std::ops::Deref;

use bevy::prelude::*;
use notify::{
    event::{MetadataKind, ModifyKind, RenameMode},
    EventKind,
};

use crate::{
    components::{DirworldEntity, Persist},
    events::{DirworldChangeRoot, DirworldEnterRoom, DirworldLeaveRoom},
    resources::{
        DirworldCache, DirworldCodecs, DirworldCurrentDir, DirworldObservers, DirworldRootDir,
    },
    utils::{despawn_entity_by_path, extract_entity_payload, spawn_entity},
    DirworldWatcherEvent,
};

/// On navigation from a room, insert modified payloads into the cache
pub fn navigate_from_room(
    trigger: Trigger<DirworldLeaveRoom>,
    entities: Query<(Entity, Ref<DirworldEntity>), Without<Persist>>,
    mut cache: ResMut<DirworldCache>,
    mut commands: Commands,
    mut event_writer: EventWriter<DirworldLeaveRoom>,
) {
    for (entity, dirworld_entity) in entities.iter() {
        if let Some(payload) = &dirworld_entity.payload {
            info!("Caching {entity:?}");
            cache.insert(dirworld_entity.path.clone(), payload.clone());
        }
        commands.entity(entity).despawn_recursive();
    }
    event_writer.send(trigger.event().clone());
}

pub fn navigate_to_room(
    trigger: Trigger<DirworldEnterRoom>,
    root_dir: Res<DirworldRootDir>,
    mut cache: ResMut<DirworldCache>,
    observers: Res<DirworldObservers>,
    codecs: Res<DirworldCodecs>,
    mut commands: Commands,
    mut event_writer: EventWriter<DirworldEnterRoom>,
    mut current_dir: ResMut<DirworldCurrentDir>,
) {
    let path = &trigger.event().0;

    let room_payload = extract_entity_payload(&path.join(".door"), &codecs).0;
    *current_dir = DirworldCurrentDir {
        path: path.to_path_buf(),
        payload: room_payload,
    };
    let entries = match path.read_dir() {
        Ok(entries) => entries
            .flatten()
            .map(|entry| entry.path().canonicalize())
            .flatten()
            .filter(|entry| {
                !entry
                    .file_name()
                    .is_some_and(|file_name| file_name.to_string_lossy().starts_with("."))
            })
            .chain(
                root_dir
                    .clone()
                    .map(|root_dir| {
                        if root_dir == *path {
                            None
                        } else {
                            Some(path.join(".."))
                        }
                    })
                    .into_iter()
                    .flatten(),
            ),
        Err(e) => {
            error!("Failed to read directory \"{}\", ({:?})", path.display(), e);
            return;
        }
    };

    for entry in entries {
        spawn_entity(&entry, &mut cache, &codecs, &observers, &mut commands);
    }
    event_writer.send(trigger.event().clone());
}

pub fn handle_changes(
    trigger: Trigger<DirworldWatcherEvent>,
    mut commands: Commands,
    dirworld_entities: Query<(Entity, &DirworldEntity)>,
    observers: Res<DirworldObservers>,
    codecs: Res<DirworldCodecs>,
    mut cache: ResMut<DirworldCache>,
    mut event_writer: EventWriter<DirworldWatcherEvent>,
) {
    let event = &trigger.event().0;
    info!("Watcher Event: {event:?}");
    match event.kind {
        EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            for path in &event.paths {
                despawn_entity_by_path(&mut commands, &dirworld_entities, path);
            }
        }
        EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
            for path in &event.paths {
                spawn_entity(path, &mut cache, &codecs, &observers, &mut commands);
            }
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            despawn_entity_by_path(&mut commands, &dirworld_entities, &event.paths[0]);
            spawn_entity(
                &event.paths[1],
                &mut cache,
                &codecs,
                &observers,
                &mut commands,
            );
        }
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)) => {
            despawn_entity_by_path(&mut commands, &dirworld_entities, &event.paths[1]);
            spawn_entity(
                &event.paths[0],
                &mut cache,
                &codecs,
                &observers,
                &mut commands,
            );
        }
        _ => {
            // warn!("Not Processed.")
        }
    }
    event_writer.send(trigger.event().clone());
}

pub fn change_root(
    trigger: Trigger<DirworldChangeRoot>,
    mut root_dir: ResMut<DirworldRootDir>,
    mut commands: Commands,
) {
    if let DirworldRootDir(Some(old_dir)) = root_dir.deref() {
        commands.trigger(DirworldLeaveRoom(old_dir.to_path_buf()));
    };

    let new_root = &trigger.event().0;
    info!("Changing Root to {}", new_root.display());
    **root_dir = Some(new_root.to_path_buf());

    commands.trigger(DirworldEnterRoom(new_root.to_path_buf()));
}
