use std::{fs, path::PathBuf};

use bevy::{
    ecs::world::{Command, CommandQueue},
    prelude::*,
    tasks::AsyncComputeTaskPool,
};
use crypto::{
    aes::KeySize,
    blockmodes::PkcsPadding,
    buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer},
};
use occule::Error;
use xz2::read::{XzDecoder, XzEncoder};

use crate::{
    payload::DirworldEntityPayload,
    resources::{DirworldCodecs, DirworldObservers, DirworldTasks},
    utils::extract_entity_payload,
    Extensions,
};

struct DirworldLockDoorCommand {
    path: PathBuf,
    key: Vec<u8>,
}

impl Command for DirworldLockDoorCommand {
    fn apply(self, world: &mut World) {
        let path = self.path.clone();
        // Get existing payload
        let codecs = world.remove_resource::<DirworldCodecs>().unwrap();
        let (payload, _) = extract_entity_payload(&path, &codecs);
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
                encrypted.extend(
                    write_buffer
                        .take_read_buffer()
                        .take_remaining()
                        .iter()
                        .map(|&i| i),
                );
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
            let mut payload = payload.unwrap_or_else(|| DirworldEntityPayload::new());
            let relationships = payload.relationships.get_or_insert_default();
            relationships.insert("key".into(), key_digest.0);

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
        let (payload, carrier) = extract_entity_payload(&path, &codecs);
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
                decrypted.extend(
                    write_buffer
                        .take_read_buffer()
                        .take_remaining()
                        .iter()
                        .map(|&i| i),
                );
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
                // Remove key relationship
                if let Some(ref mut relationships) = payload.relationships {
                    relationships.remove("key");
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
    /// Lock Door
    fn dirworld_lock_door(&mut self, path: PathBuf, key: Vec<u8>);

    /// Unlock Door
    fn dirworld_unlock_door(&mut self, path: PathBuf, key: Vec<u8>);

    /// Save entity
    fn dirworld_save_entity(&mut self, path: PathBuf, payload: DirworldEntityPayload);
}

impl<'w, 's> DirworldCommands for Commands<'w, 's> {
    fn dirworld_lock_door(&mut self, path: PathBuf, key: Vec<u8>) {
        self.queue(DirworldLockDoorCommand { key, path });
    }

    fn dirworld_unlock_door(&mut self, path: PathBuf, key: Vec<u8>) {
        self.queue(DirworldUnlockDoorCommand { key, path });
    }

    fn dirworld_save_entity(&mut self, path: PathBuf, payload: DirworldEntityPayload) {
        self.queue(DirworldSaveEntityCommand { path, payload });
    }
}
