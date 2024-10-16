use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future},
};

use crate::{components::DirworldEntity, payload::DirworldComponent, resources::DirworldTasks};

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

pub fn sync_entity_transforms(
    mut dirworld_entity_query: Query<(&mut DirworldEntity, Ref<Transform>, &GlobalTransform)>,
) {
    for (mut dirworld_entity, transform, global_transform) in dirworld_entity_query.iter_mut() {
        if transform.is_changed() && !transform.is_added() {
            if let Some(payload) = &mut dirworld_entity.payload {
                if let Some(DirworldComponent::Transform(payload_transform)) =
                    payload.component_mut("Transform")
                {
                    let transform = global_transform.compute_transform();
                    *payload_transform = transform;
                }
            }
        }
    }
}
