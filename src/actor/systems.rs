use bevy::prelude::*;
use bevy_basic_interaction::events::InteractionEvent;
use yarnspinner::core::YarnValue;

use super::{
    components::Actor,
    events::{ContinueDialogueEvent, DialogueEvent}, DIRWORLD_VARIABLE_STORAGE,
};

pub fn handle_dialog_initiation(
    mut event_reader: EventReader<InteractionEvent>,
    mut actor_query: Query<(Entity, &mut Actor)>,
    mut event_writer: EventWriter<ContinueDialogueEvent>,
) {
    for InteractionEvent { interactable, .. } in event_reader.read() {
        if let Ok((actor_entity, mut actor)) = actor_query.get_mut(*interactable) {
            actor.active = true;
            event_writer.send(ContinueDialogueEvent::Continue(actor_entity));
        }
    }
}

pub fn progress_dialog(
    mut event_reader: EventReader<ContinueDialogueEvent>,
    mut actor_query: Query<&mut Actor>,
    mut event_writer: EventWriter<DialogueEvent>,
) {
    for event in event_reader.read() {
        let actor_entity = match event {
            ContinueDialogueEvent::Continue(actor) => actor,
            ContinueDialogueEvent::SelectedOption { actor, .. } => actor,
        };

        if let Ok(mut actor) = actor_query.get_mut(*actor_entity) {
            if let ContinueDialogueEvent::SelectedOption { option, .. } = event {
                actor.dialogue.set_selected_option(*option).unwrap();
            }
            if actor.dialogue.current_node().is_none() {
                actor.dialogue.set_node("Start").unwrap();
            }
            match actor.dialogue.continue_() {
                Ok(events) => {
                    info!("BATCH");
                    for event in events {
                        info!("Event: {:?}", event);
                        match event {
                            yarnspinner::prelude::DialogueEvent::Line(line) => {
                                event_writer.send(DialogueEvent::Line {
                                    actor: *actor_entity,
                                    line,
                                });
                            }
                            yarnspinner::prelude::DialogueEvent::DialogueComplete => {
                                event_writer.send(DialogueEvent::DialogueComplete {
                                    actor: *actor_entity,
                                });
                            }
                            yarnspinner::prelude::DialogueEvent::Options(options) => {
                                event_writer.send(DialogueEvent::Options {
                                    actor: *actor_entity,
                                    options,
                                });
                            }
                            yarnspinner::runtime::DialogueEvent::Command(command) => {
                                event_writer.send(DialogueEvent::Command {
                                    actor: *actor_entity,
                                    command,
                                });
                            }
                            yarnspinner::runtime::DialogueEvent::NodeStart(name) => {
                                event_writer.send(DialogueEvent::NodeStart {
                                    actor: *actor_entity,
                                    name,
                                });
                            }
                            yarnspinner::runtime::DialogueEvent::NodeComplete(name) => {
                                event_writer.send(DialogueEvent::NodeComplete {
                                    actor: *actor_entity,
                                    name,
                                });
                            }
                            yarnspinner::runtime::DialogueEvent::LineHints(lines) => {
                                event_writer.send(DialogueEvent::LineHints {
                                    actor: *actor_entity,
                                    lines,
                                });
                            }
                        }
                    }
                }
                Err(err) => error!("{:?}", err),
            }
        }
    }
}

pub fn handle_variable_set_commands(
    mut event_reader: EventReader<DialogueEvent>,
    mut event_writer: EventWriter<ContinueDialogueEvent>,
) {
    for event in event_reader.read() {
        if let DialogueEvent::Command { command, actor } = event {
            if command.name != "set_var" {
                continue;
            }

            event_writer.send(ContinueDialogueEvent::Continue(*actor));

            if command.parameters.len() != 3 {
                warn!("Incorrect number of parameters passed to set command: {}", command.parameters.len());
                continue;
            }

            if let YarnValue::String(instance_name) = &command.parameters[0] {
                if let YarnValue::String(var_name) = &command.parameters[1] {
                    DIRWORLD_VARIABLE_STORAGE.lock().unwrap().set(instance_name, var_name, command.parameters[2].clone());
                }
            }
        }
    }
}
