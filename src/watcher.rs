use std::{path::{Path, PathBuf}, time::Duration};

use async_channel::{Receiver, Sender};
use bevy::{prelude::*, tasks::IoTaskPool};
use notify::{
    event::{AccessKind, AccessMode, DataChange, MetadataKind, ModifyKind, RenameMode},
    EventKind, RecursiveMode, Watcher,
};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};

use crate::{
    commands::process_entry,
    components::DirworldEntity,
    resources::{DirworldCodecs, DirworldObservers, DirworldRootDir},
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirworldWatcherSet;

/// Event fired when a file watcher event is caught.
#[derive(Event)]
pub struct DirworldWatcherEvent(pub notify::Event);

#[derive(Resource)]
pub struct WatcherChannels {
    tx_control: Sender<PathBuf>,
    rx_changes: Receiver<notify::Event>,
}

pub fn setup(mut commands: Commands) {
    let (tx_control, rx_control) = async_channel::unbounded();
    let (tx_changes, rx_changes) = async_channel::unbounded();
    IoTaskPool::get()
        .spawn(async move { file_watcher(rx_control, tx_changes).await })
        .detach();

    commands.insert_resource(WatcherChannels {
        tx_control,
        rx_changes,
    })
}

async fn file_watcher(rx: Receiver<PathBuf>, tx: Sender<notify::Event>) {
    let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), None, move |result: DebounceEventResult| {
        match result {
            Ok(events) => for event in events.iter() {
                watcher_tx.send(event.clone()).unwrap();
            }
            Err(errors) => for error in errors.iter() {
                error!("{error:?}");
            }
        }
    }).unwrap();
    let mut old_path: Option<PathBuf> = None;
    loop {
        while let Ok(message) = rx.try_recv() {
            if let Some(old_path) = &old_path {
                debouncer.watcher().unwatch(old_path).unwrap();
            }
            debouncer.watcher().watch(&message, RecursiveMode::NonRecursive).unwrap();
            old_path = Some(message);
        }

        while let Ok(event) = watcher_rx.try_recv() {
            tx.send(event.event.clone()).await.unwrap();
        }
    }
}

pub fn update(
    watcher_channels: Res<WatcherChannels>,
    mut event_writer: EventWriter<DirworldWatcherEvent>,
    root_dir: Res<DirworldRootDir>,
) {
    if root_dir.is_changed() {
        if let Some(project_dir) = &root_dir.0 {
            let _ = watcher_channels.tx_control.try_send(project_dir.clone());
        }
    } else {
        while let Ok(event) = watcher_channels.rx_changes.try_recv() {
            event_writer.send(DirworldWatcherEvent(event));
        }
    }
}

pub fn handle_changes(
    mut event_reader: EventReader<DirworldWatcherEvent>,
    mut commands: Commands,
    dirworld_entities: Query<(Entity, &DirworldEntity)>,
    observers: Res<DirworldObservers>,
    codecs: Res<DirworldCodecs>,
) {
    if !event_reader.is_empty() {
        for DirworldWatcherEvent(event) in event_reader.read() {
            info!("Watcher Event: {event:?}");
            match event.kind {
                EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From))  => {
                    for path in &event.paths {
                        remove_entity(&mut commands, &dirworld_entities, path);
                    }
                }
                EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    for path in &event.paths {
                        process_entry(&mut commands, path, &observers, &codecs);
                    }
                }
                EventKind::Modify(ModifyKind::Name(RenameMode::Both)) 
                => {
                    remove_entity(&mut commands, &dirworld_entities, &event.paths[0]);
                    process_entry(&mut commands, &event.paths[1], &observers, &codecs);
                }
                // EventKind::Modify(ModifyKind::Data(DataChange::Content))
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)) => {
                    remove_entity(&mut commands, &dirworld_entities, &event.paths[0]);
                    process_entry(&mut commands, &event.paths[0], &observers, &codecs);
                }
                _ => {
                    // warn!("Not Processed.")
                }
            }
        }
    }
}

fn remove_entity(
    commands: &mut Commands,
    dirworld_entities: &Query<(Entity, &DirworldEntity)>,
    path: &Path,
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
