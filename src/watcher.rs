use std::{
    path::PathBuf,
    time::Duration,
};

use async_channel::{Receiver, Sender};
use bevy::{prelude::*, tasks::IoTaskPool};
use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};

use crate::resources::DirworldRootDir;

/// SystemSet for dirworld watcher systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirworldWatcherSet;

/// Event fired when a file watcher event is caught.
#[derive(Event, Clone, Debug)]
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
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => {
                for event in events.iter() {
                    watcher_tx.send(event.clone()).unwrap();
                }
            }
            Err(errors) => {
                for error in errors.iter() {
                    error!("{error:?}");
                }
            }
        },
    )
    .unwrap();
    let mut old_path: Option<PathBuf> = None;
    loop {
        while let Ok(message) = rx.try_recv() {
            if let Some(old_path) = &old_path {
                debouncer.unwatch(old_path).unwrap();
            }
            debouncer
                .watch(&message, RecursiveMode::NonRecursive)
                .unwrap();
            old_path = Some(message);
        }

        while let Ok(event) = watcher_rx.try_recv() {
            tx.send(event.event.clone()).await.unwrap();
        }
    }
}

pub fn update(
    watcher_channels: Res<WatcherChannels>,
    root_dir: Res<DirworldRootDir>,
    mut commands: Commands,
) {
    if root_dir.is_changed() {
        if let Some(project_dir) = &root_dir.0 {
            let _ = watcher_channels.tx_control.try_send(project_dir.clone());
        }
    } else {
        while let Ok(event) = watcher_channels.rx_changes.try_recv() {
            commands.trigger(DirworldWatcherEvent(event));
        }
    }
}
