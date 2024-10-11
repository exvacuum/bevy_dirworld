use std::{
    fs, iter,
    path::{Path, PathBuf},
};

use bevy::{
    ecs::{
        system::SystemState,
        world::{Command, CommandQueue},
    },
    prelude::*,
    tasks::AsyncComputeTaskPool,
};
use crypto::{
    aes::KeySize,
    blockmodes::{EcbEncryptor, PkcsPadding},
    buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer},
};
use occule::Error;
use xz2::read::{XzDecoder, XzEncoder};

use crate::{
    components::DirworldEntity,
    events::{DirworldNavigationEvent, DirworldSpawn},
    payload::{DirworldComponent, DirworldComponentDiscriminants, DirworldEntityPayload},
    resources::{
        DirworldCodecs, DirworldCurrentDir, DirworldObservers, DirworldRootDir, DirworldTasks,
        EntryType,
    },
    Extensions,
};

struct DirworldNavigateCommand {
    pub path: PathBuf,
}

impl Command for DirworldNavigateCommand {
    fn apply(self, world: &mut World) {
        let root_dir = world.remove_resource::<DirworldRootDir>().unwrap();
        let mut current_dir = world.remove_resource::<DirworldCurrentDir>().unwrap();

        let current_path;
        let old_dir;
        if let Some(old_path) = &current_dir.0 {
            world.send_event(DirworldNavigationEvent::LeftRoom {
                path: old_path.clone(),
            });

            current_path = old_path.join(self.path);
            old_dir = Some(old_path.clone());
        } else {
            current_path = self.path;
            old_dir = None;
        }
        current_dir.0 = Some(current_path.clone());

        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, &DirworldEntity)>,
            Res<DirworldObservers>,
            Res<DirworldCodecs>,
        )> = SystemState::new(world);
        let (mut commands, dirworld_entities, observers, codecs) = system_state.get_mut(world);
        update_entries(
            &mut commands,
            &dirworld_entities,
            old_dir,
            &current_path,
            &root_dir.0.clone().unwrap(),
            &observers,
            &codecs,
        );
        system_state.apply(world);

        world.send_event(DirworldNavigationEvent::EnteredRoom { path: current_path });
        world.insert_resource(current_dir);
        world.insert_resource(root_dir);
    }
}

pub(crate) fn update_entries(
    commands: &mut Commands,
    dirworld_entities: &Query<(Entity, &DirworldEntity)>,
    old_dir: Option<PathBuf>,
    current_dir: &PathBuf,
    project_dir: &PathBuf,
    observers: &DirworldObservers,
    codecs: &DirworldCodecs,
) {
    let directory = current_dir.read_dir().unwrap();

    if let Some(old_dir) = old_dir {
        let mut entities_to_despawn = vec![];
        for (entity, dirworld_entity) in dirworld_entities.iter() {
            if dirworld_entity.path.parent().unwrap() == old_dir {
                entities_to_despawn.push(entity);
            }
        }
        for entity in entities_to_despawn {
            commands.entity(entity).despawn_recursive();
        }
    }

    let mut entry_paths: Vec<PathBuf> = directory
        .flatten()
        .map(|entry| entry.path().canonicalize().unwrap())
        .collect::<Vec<_>>();
    entry_paths.retain(|entry| {
        !entry
            .file_name()
            .is_some_and(|entry| entry.to_string_lossy().starts_with("."))
    });
    if current_dir != project_dir {
        entry_paths = iter::once(current_dir.join(".."))
            .chain(entry_paths)
            .collect();
    }

    for entry_path in entry_paths {
        process_entry(commands, &entry_path, &observers, &codecs);
    }
}

pub(crate) fn process_entry(
    commands: &mut Commands,
    entry_path: &PathBuf,
    observers: &DirworldObservers,
    codecs: &DirworldCodecs,
) {
    let (payload, data) = extract_payload(entry_path, codecs);
    let transform = if let Some(component) = payload
        .as_ref()
        .and_then(|payload| payload.component("Transform"))
    {
        if let DirworldComponent::Transform(component) = component {
            component.clone()
        } else {
            panic!("Failed to decompose component")
        }
    } else {
        Transform::default()
    };

    let entity = commands.spawn((
        SpatialBundle {
            transform,
            ..Default::default()
        },
        DirworldEntity {
            path: entry_path.clone(),
            payload: payload.clone(),
        },
    ));

    let entity = entity.id();
    let entry_type = if entry_path.is_dir() {
        EntryType::Folder
    } else {
        let extensions = entry_path.extensions();
        EntryType::File(extensions)
    };
    if let Some(observer) = observers.get(&entry_type) {
        commands.trigger_targets(DirworldSpawn { entity, data }, observer.clone());
    }
}

fn extract_payload(
    entry_path: &PathBuf,
    codecs: &DirworldCodecs,
) -> (Option<DirworldEntityPayload>, Option<Vec<u8>>) {
    let entry_type = if entry_path.is_dir() {
        EntryType::Folder
    } else {
        let extensions = entry_path.extensions();
        EntryType::File(extensions)
    };

    let mut data: Option<Vec<u8>> = None;
    let mut payload: Option<DirworldEntityPayload> = None;
    match &entry_type {
        EntryType::File(Some(extension)) => {
            if let Ok(file_data) = fs::read(entry_path.clone()) {
                match codecs.get(extension) {
                    Some(codec) => match codec.decode(&file_data.as_slice()) {
                        Ok((carrier, extracted_payload)) => {
                            match rmp_serde::from_slice::<DirworldEntityPayload>(
                                extracted_payload.as_slice(),
                            ) {
                                Ok(deserialized_payload) => {
                                    data = Some(carrier);
                                    payload = Some(deserialized_payload);
                                }
                                Err(e) => {
                                    warn!("{:?}", e);
                                    data = Some(file_data);
                                }
                            }
                        }
                        Err(e) => match e {
                            Error::DataNotEncoded => {
                                data = Some(file_data);
                            }
                            _ => error!("{:?}", e),
                        },
                    },
                    None => {
                        data = Some(file_data);
                    }
                }
            } else {
                warn!("Failed to read data from {entry_path:?}");
            }
        }
        EntryType::Folder => {
            let door_path = entry_path.join(".door");
            if door_path.exists() {
                let door_file_data = fs::read(door_path).unwrap();
                match rmp_serde::from_slice::<DirworldEntityPayload>(&door_file_data.as_slice()) {
                    Ok(deserialized_payload) => {
                        payload = Some(deserialized_payload);
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                }
            }
        }
        _ => {}
    }
    (payload, data)
}

struct DirworldChangeRootCommand {
    pub path: PathBuf,
}

impl Command for DirworldChangeRootCommand {
    fn apply(self, world: &mut World) {
        let mut root_dir = world.remove_resource::<DirworldRootDir>().unwrap();
        let mut current_dir = world.remove_resource::<DirworldCurrentDir>().unwrap();

        let old_root;
        if let DirworldRootDir(Some(old_dir)) = root_dir {
            world.send_event(DirworldNavigationEvent::LeftRoom {
                path: self.path.clone(),
            });
            old_root = Some(old_dir);
        } else {
            old_root = None;
        }

        root_dir.0 = Some(self.path.canonicalize().unwrap());
        current_dir.0 = Some(self.path.canonicalize().unwrap());

        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, &DirworldEntity)>,
            Res<DirworldObservers>,
            Res<DirworldCodecs>,
        )> = SystemState::new(world);
        let (mut commands, dirworld_entities, observers, codecs) = system_state.get_mut(world);
        update_entries(
            &mut commands,
            &dirworld_entities,
            old_root,
            &current_dir.0.clone().unwrap(),
            &root_dir.0.clone().unwrap(),
            &observers,
            &codecs,
        );
        system_state.apply(world);

        world.send_event(DirworldNavigationEvent::EnteredRoom { path: self.path });

        world.insert_resource(root_dir);
        world.insert_resource(current_dir);
    }
}

struct DirworldLockDoorCommand {
    path: PathBuf,
    key: Vec<u8>,
}

impl Command for DirworldLockDoorCommand {
    fn apply(self, world: &mut World) {
        let path = self.path.clone();
        // Get existing payload
        let codecs = world.remove_resource::<DirworldCodecs>().unwrap();
        let (payload, _) = extract_payload(&path, &codecs);
        world.insert_resource(codecs);
        let task = AsyncComputeTaskPool::get().spawn(async move {
            // Tar directory
            let mut tar = tar::Builder::new(Vec::new());
            tar.append_dir_all(path.file_stem().unwrap(), path.clone())
                .unwrap();
            let tar_buffer = tar.into_inner().unwrap();

            // XZ archive
            let tar_xz = XzEncoder::new(tar_buffer.as_slice(), 0).into_inner();

            // Encrypt archive
            let mut crypter =
                crypto::aes::ecb_encryptor(KeySize::KeySize128, &self.key[..16], PkcsPadding);
            let mut encrypted = vec![];
            let mut buffer = [0; 4096];

            let mut read_buffer = RefReadBuffer::new(tar_xz);
            let mut write_buffer = RefWriteBuffer::new(&mut buffer);
            loop {
                let result = crypter
                    .encrypt(&mut read_buffer, &mut write_buffer, true)
                    .expect("Failed to encrypt data!");
                encrypted.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i|i));
                match result {
                    BufferResult::BufferUnderflow => break,
                    BufferResult::BufferOverflow => {}
                }
            }

            let newpath = format!("{}.tar.xz.aes", path.display());
            fs::write(&newpath, encrypted).unwrap();

            // Remove original folder
            fs::remove_dir_all(path).unwrap();

            // Insert key hash as payload relationship
            let key_digest = md5::compute(&self.key[..16]);
            let mut payload = payload.unwrap_or_default();
            payload.push(DirworldComponent::Relationship {
                label: "key".into(),
                hash: key_digest.0,
            });

            // Write payload
            let mut command_queue = CommandQueue::default();
            command_queue.push(DirworldSaveEntityCommand {
                path: newpath.into(),
                payload,
            });
            Some(command_queue)
        });
        world.resource_mut::<DirworldTasks>().insert(
            format!("Locking {:?}", self.path.file_name().unwrap()),
            task,
        );
    }
}

struct DirworldUnlockDoorCommand {
    path: PathBuf,
    key: Vec<u8>,
}

impl Command for DirworldUnlockDoorCommand {
    fn apply(self, world: &mut World) {
        let path = self.path.clone();
        // Get existing payload
        let codecs = world.remove_resource::<DirworldCodecs>().unwrap();
        let (payload, carrier) = extract_payload(&path, &codecs);
        world.insert_resource(codecs);
        let task = AsyncComputeTaskPool::get().spawn(async move {
            // Decrypt archive
            let mut decrypter =
                crypto::aes::ecb_decryptor(KeySize::KeySize128, &self.key[..16], PkcsPadding);
            let encrypted = carrier.unwrap();
            let mut decrypted = vec![];
            let mut buffer = [0; 4096];

            let mut read_buffer = RefReadBuffer::new(&encrypted);
            let mut write_buffer = RefWriteBuffer::new(&mut buffer);
            loop {
                let result = decrypter
                    .decrypt(&mut read_buffer, &mut write_buffer, true)
                    .expect("Failed to encrypt data!");
                decrypted.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i|i));
                match result {
                    BufferResult::BufferUnderflow => break,
                    BufferResult::BufferOverflow => {}
                }
            }

            // Unzip archive
            let tar = XzDecoder::new(decrypted.as_slice()).into_inner();

            // Untar archive
            let mut tar = tar::Archive::new(tar);
            let parent = path.parent().unwrap();
            tar.unpack(parent).unwrap();

            fs::remove_file(path.clone()).unwrap();

            if let Some(mut payload) = payload {
                for (index, relationship) in payload.iter().enumerate().filter(|(_, x)| {
                    DirworldComponentDiscriminants::from(*x)
                        == DirworldComponentDiscriminants::Relationship
                }) {
                    if let DirworldComponent::Relationship { label, .. } = relationship {
                        if label == "key" {
                            payload.remove(index);
                            break;
                        }
                    }
                }

                // Write payload
                let mut command_queue = CommandQueue::default();
                let new_path = parent.join(path.file_stem_no_extensions().unwrap());
                let _ = fs::create_dir(new_path.clone());
                command_queue.push(DirworldSaveEntityCommand {
                    path: new_path.into(),
                    payload,
                });
                return Some(command_queue);
            }
            None
        });
        world.resource_mut::<DirworldTasks>().insert(
            format!("Unlocking {:?}", self.path.file_name().unwrap()),
            task,
        );
    }
}

struct DirworldSaveEntityCommand {
    path: PathBuf,
    payload: DirworldEntityPayload,
}

impl Command for DirworldSaveEntityCommand {
    fn apply(self, world: &mut World) {
        info!("Saving {}", &self.path.display());
        let is_dir = self.path.is_dir();
        let observers = world.remove_resource::<DirworldObservers>().unwrap();
        let codecs = world.remove_resource::<DirworldCodecs>().unwrap();
        let codec = if is_dir {
            None
        } else {
            match codecs.get(&self.path.extensions().unwrap()) {
                Some(codec) => Some(codec),
                None => {
                    warn!(
                        "No matching codec found for {:?}",
                        self.path.file_name().unwrap()
                    );
                    world.insert_resource(codecs);
                    world.insert_resource(observers);
                    return;
                }
            }
        };

        let payload = match rmp_serde::to_vec(&self.payload) {
            Ok(payload) => payload,
            Err(e) => {
                error!("{e:?}");
                world.insert_resource(codecs);
                world.insert_resource(observers);
                return;
            }
        };

        if is_dir {
            let target_path = self.path.join(".door");
            if let Err(e) = fs::write(target_path, payload) {
                error!("{e:?}");
            }
        } else {
            let codec = codec.unwrap();
            let carrier = match fs::read(&self.path) {
                Ok(raw_carrier) => match codec.decode(&raw_carrier) {
                    Ok((carrier, _)) => carrier,
                    Err(e) => match e {
                        Error::DependencyError(_) => {
                            error!("{e:?}");
                            world.insert_resource(codecs);
                            world.insert_resource(observers);
                            return;
                        }
                        _ => raw_carrier,
                    },
                },
                Err(e) => {
                    error!("{e:?}");
                    world.insert_resource(codecs);
                    world.insert_resource(observers);
                    return;
                }
            };

            let encoded = match codec.encode(&carrier, &payload) {
                Ok(encoded) => encoded,
                Err(e) => {
                    error!("Error encoding payload: {e:?}");
                    world.insert_resource(codecs);
                    world.insert_resource(observers);
                    return;
                }
            };
            if let Err(e) = fs::write(&self.path, encoded) {
                error!("{e:?}");
            }
        }

        world.insert_resource(codecs);
        world.insert_resource(observers);
    }
}

/// Commands for dirworld navigation
pub trait DirworldCommands {
    /// Change the root of the world. This will also set the current directory. This is not really meant to be used in-game but is useful for editor applications.
    fn dirworld_change_root(&mut self, path: PathBuf);

    /// Move to given directory
    fn dirworld_navigate(&mut self, path: PathBuf);

    /// Lock Door
    fn dirworld_lock_door(&mut self, path: PathBuf, key: Vec<u8>);

    /// Unlock Door
    fn dirworld_unlock_door(&mut self, path: PathBuf, key: Vec<u8>);

    fn dirworld_save_entity(&mut self, path: PathBuf, payload: DirworldEntityPayload);
}

impl<'w, 's> DirworldCommands for Commands<'w, 's> {
    fn dirworld_change_root(&mut self, path: PathBuf) {
        self.add(DirworldChangeRootCommand { path });
    }

    fn dirworld_navigate(&mut self, path: PathBuf) {
        self.add(DirworldNavigateCommand { path });
    }

    fn dirworld_lock_door(&mut self, path: PathBuf, key: Vec<u8>) {
        self.add(DirworldLockDoorCommand { key, path });
    }

    fn dirworld_unlock_door(&mut self, path: PathBuf, key: Vec<u8>) {
        self.add(DirworldUnlockDoorCommand { key, path });
    }

    fn dirworld_save_entity(&mut self, path: PathBuf, payload: DirworldEntityPayload) {
        self.add(DirworldSaveEntityCommand { path, payload });
    }
}
