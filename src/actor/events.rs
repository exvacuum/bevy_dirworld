//! Actor-related events

use bevy::prelude::*;
use yarnspinner::{core::LineId, runtime::{Command, DialogueOption, Line, OptionId}};

/// Event called by user to progress dialogue
#[derive(Debug, Event)]
pub enum ContinueDialogueEvent {
    /// Continue to next line of dialogue for given actor entity
    Continue(Entity),
    /// Submit option selection to given actor entity
    SelectedOption { 
        /// Target actor entity
        actor: Entity,
        /// Selected option ID
        option: OptionId 
    },
}

/// Event called by plugin in response to a corresponding yarnspinner dialogue events
///
/// The user should catch these events to update UI, and never call it directly.
#[derive(Event)]
pub enum DialogueEvent {
    /// Recieved new line of dialogue
    Line {
        /// Actor entity
        actor: Entity,
        /// Line of dialogue received
        line: Line,
    },
    /// Dialogue complete
    DialogueComplete {
        /// Actor entity
        actor: Entity,
    },
    /// Encountered an option selection
    Options {
        /// Actor entity
        actor: Entity,
        /// Options to select from
        options: Vec<DialogueOption>,
    },
    /// Triggered a yarnspinner command
    Command {
        /// Actor entity
        actor: Entity,
        /// Triggered command
        command: Command,
    },
    /// Node started
    NodeStart {
        /// Actor entity
        actor: Entity,
        /// Name of started node
        name: String,
    },
    /// Node complete
    NodeComplete {
        /// Actor entity
        actor: Entity,
        /// Name of completed node
        name: String,
    },
    /// Received line hints
    LineHints {
        /// Actor entity
        actor: Entity,
        /// Lines affected
        lines: Vec<LineId>,
    },
}
