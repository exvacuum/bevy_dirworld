use std::{fs, path::PathBuf};

use bevy::prelude::*;

use crate::{
    components::DirworldEntity, events::DirworldSpawn, payload::{DirworldComponent, DirworldEntityPayload}, resources::{DirworldCache, DirworldCodecs, DirworldObservers, EntryType}, Extensions
};

pub fn extract_entity_payload(
    path: &PathBuf,
    codecs: &DirworldCodecs,
) -> (Option<DirworldEntityPayload>, Option<Vec<u8>>) {
    let mut data = None;
    let mut payload = None;

    if path.is_dir() {
        let payload_file_path = path.join(".door");
        if payload_file_path.exists() {
            if let Ok(payload_file_data) = fs::read(&payload_file_path) {
                match rmp_serde::from_slice::<DirworldEntityPayload>(&payload_file_data) {
                    Ok(deserialized_payload) => {
                        payload = Some(deserialized_payload);
                    }
                    Err(e) => {
                        warn!("Could not deserialize extracted payload: {e:?}");
                    }
                }
            }
        }
    } else {
        if let Some(extensions) = path.extensions() {
            if let Ok(file_data) = fs::read(&path) {
                if let Some(codec) = codecs.get(&extensions) {
                    match codec.decode(&file_data) {
                        Ok((carrier, extracted_payload)) => {
                            match rmp_serde::from_slice::<DirworldEntityPayload>(&extracted_payload)
                            {
                                Ok(deserialized_payload) => {
                                    data = Some(carrier);
                                    payload = Some(deserialized_payload);
                                }
                                Err(e) => {
                                    warn!("Could not deserialize extracted payload: {e:?}");
                                    data = Some(file_data);
                                }
                            }
                        }
                        Err(e) => match e {
                            occule::Error::DataNotEncoded => {
                                data = Some(file_data);
                            }
                            _ => error!("Could not decode payload: {e:?}"),
                        },
                    }
                } else {
                    data = Some(file_data);
                }
            }
        }
    }

    (payload, data)
}

pub fn spawn_entity(
    entry: &PathBuf,
    cache: &mut DirworldCache,
    codecs: &DirworldCodecs,
    observers: &DirworldObservers,
    commands: &mut Commands,
) {
    let (mut payload, data) = extract_entity_payload(&entry, &codecs);
    if let Some(cached_payload) = cache.remove(entry) {
        payload = Some(cached_payload);
    }

    let transform = if let Some(component) = payload
        .as_ref()
        .and_then(|payload| payload.component("Transform"))
    {
        if let DirworldComponent::Transform(transform) = component {
            transform.clone()
        } else {
            panic!("BAD DECOMPOSE: TRANSFORM ({component:?})");
        }
    } else {
        Transform::default()
    };
    let entry_type = if entry.is_dir() {
        EntryType::Folder
    } else {
        EntryType::File(entry.extensions())
    };
    let entity = commands
        .spawn((
            SpatialBundle {
                transform,
                ..Default::default()
            },
            DirworldEntity {
                path: entry.clone(),
                payload,
            },
        ))
        .id();
    if let Some(observer) = observers.get(&entry_type) {
        commands.trigger_targets(DirworldSpawn { entity, data }, observer.clone());
    }
}

pub fn despawn_entity_by_path(
    commands: &mut Commands,
    dirworld_entities: &Query<(Entity, &DirworldEntity)>,
    path: &PathBuf,
) {
    if let Some((entity, _)) = dirworld_entities
        .iter()
        .find(|(_, dirworld_entity)| dirworld_entity.path == *path)
    {
        commands.entity(entity).despawn_recursive();
    } else {
        warn!("Failed to find entity corresponding to path for despawning: {path:?}");
    }
}
