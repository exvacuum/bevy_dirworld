use std::{fs, path::PathBuf};

use bevy::prelude::*;

use crate::{
    components::DirworldEntity, payload::DirworldEntityPayload, resources::DirworldCodecs, Extensions
};

/// Extracts the binary payload from a file
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

/// Despawns an entity corresponding to a path on the filesystem
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
