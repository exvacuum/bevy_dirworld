use bevy::{prelude::{Commands, ResMut}, tasks::{block_on, futures_lite::future}};

use crate::resources::DirworldTasks;

pub fn remove_completed_tasks(mut commands: Commands, mut tasks: ResMut<DirworldTasks>) {
    tasks.retain(|_, task| {
        if task.is_finished() {
            if let Some(Some(mut command_queue)) = block_on(future::poll_once(&mut *task)) {
                commands.append(&mut command_queue);
            }
        }
        !task.is_finished()
    });
}
