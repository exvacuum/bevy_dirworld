//! Components related to actors

use bevy::{prelude::*, utils::HashMap};
use yarnspinner::{compiler::{Compiler, File}, core::{Library, LineId}, runtime::{Dialogue, MemoryVariableStorage, StringTableTextProvider}};

/// Main actor component, holds state about dialogue along with the dialogue runner itself
#[derive(Component)]
pub struct Actor {
    /// Whether this actor is currently conversing
    pub active: bool,
    /// Yarnspinner dialogue runner
    pub dialogue: Dialogue,
    /// Yarnspinner dialogue metadata
    pub metadata: HashMap<LineId, Vec<String>>,
}

impl Actor {
    /// Create a new actor from the given source code, starting on the given start node, and with
    /// the given function library
    pub fn new(file_name: &str, source: &[u8], start_node: &str, function_library: &Library) -> Self {
        let compilation = Compiler::new()
            .add_file(File {
                source: String::from_utf8_lossy(source).into(),
                file_name: file_name.into(),
            })
            .compile()
            .unwrap();

        let mut base_language_string_table = std::collections::HashMap::new();
        let mut metadata = HashMap::new();

        for (k, v) in compilation.string_table {
            base_language_string_table.insert(k.clone(), v.text);
            metadata.insert(k, v.metadata);
        }

        let mut text_provider = StringTableTextProvider::new();
        text_provider.extend_base_language(base_language_string_table);

        let mut dialogue = Dialogue::new(Box::new(MemoryVariableStorage::new()), Box::new(text_provider));
        dialogue.library_mut().extend(function_library.clone());
        dialogue.add_program(compilation.program.unwrap());
        dialogue.set_node(start_node).unwrap();

        Self {
            active: false,
            dialogue,
            metadata,
        }
    }
}

