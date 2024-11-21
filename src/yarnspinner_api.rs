use bevy::{log::info, prelude::World};
use lazy_static::lazy_static;
use std::sync::{
    Arc, Condvar, Mutex,
};

use crate::{actor::resources::FunctionLibrary, conditionals::Condition};

lazy_static! {
    static ref YARNSPINNER_WORLD: Arc<Mutex<World>> = Arc::new(Mutex::new(World::new()));
    static ref YARNSPINNER_CVAR: Arc<(Condvar, Mutex<bool>)> =
        Arc::new((Condvar::new(), Mutex::new(false)));
    static ref YARNSPINNER_COMMAND_COUNT: Arc<(Condvar, Mutex<usize>)> =
        Arc::new((Condvar::new(), Mutex::new(0)));
}

macro_rules! register_fns {
    ($runtime:expr, $($function:expr),+) => {
        {
            $runtime$(.add_function(stringify!($function).to_string(), $function))+
        }
    };
}

pub fn setup_yarnspinner_functions(library: &mut FunctionLibrary) {
    register_fns!(library, conditional_true);
}

pub fn process_commands(world: &mut World) {
    let command_count = YARNSPINNER_COMMAND_COUNT.1.lock().unwrap();
    if *command_count <= 0 { 
        return;
    }

    info!("Swapping World to Yarnspinner");
    let mut temp_world = World::new();
    std::mem::swap(&mut temp_world, world);
    {
        let mut world_swapped = YARNSPINNER_CVAR.1.lock().unwrap();
        *world_swapped = true;
    }

    let mut command_count = YARNSPINNER_COMMAND_COUNT.1.lock().unwrap();
    while !*command_count <= 0 {
        info!("Command Count: {}", *command_count);
        command_count = YARNSPINNER_COMMAND_COUNT.0.wait(command_count).unwrap();
    }

    info!("Swapping World from Yarnspinner");
    std::mem::swap(&mut temp_world, world);
    {
        let mut world_swapped = YARNSPINNER_CVAR.1.lock().unwrap();
        *world_swapped = false;
    }
}

fn conditional(condition: Condition) -> bool {
    {
        let mut command_count = YARNSPINNER_COMMAND_COUNT.1.lock().unwrap();
        *command_count += 1;
    }

    let mut world_swapped = YARNSPINNER_CVAR.1.lock().unwrap();
    while !*world_swapped {
        world_swapped = YARNSPINNER_CVAR.0.wait(world_swapped).unwrap();
    }

    let result = condition.evaluate(&mut YARNSPINNER_WORLD.lock().unwrap());

    {
        let mut command_count = YARNSPINNER_COMMAND_COUNT.1.lock().unwrap();
        *command_count -= 1;
    }
    result
}

fn conditional_true() -> bool {
    conditional(Condition::True)
}
