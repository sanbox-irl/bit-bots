#![warn(elided_lifetimes_in_paths)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate bit_bots_derive;
#[macro_use]
extern crate anyhow;

mod action_map;
mod clockwork;
mod components;
mod components_singleton;
mod ecs;
mod entities;
mod hardware_interfaces;
mod physics;
mod resources;
mod scene;
mod scene_graph;
mod serialization;
mod systems;
mod tick_structs;
mod utilities;

pub use action_map::ActionMap;
pub use clockwork::*;
pub use components::*;
pub use components_singleton::*;
pub use ecs::*;
pub use entities::*;
pub use hardware_interfaces::*;
pub use physics::*;
pub use resources::*;
pub use scene::*;
pub use scene_graph::*;
pub use serialization::*;
pub use systems::*;
pub use tick_structs::*;
pub use utilities::*;

fn main() {
    pretty_env_logger::init();

    let mut clockwork = match clockwork::Clockwork::new() {
        Ok(clockwork) => clockwork,
        Err(e) => {
            error!("Error on Startup: {}", e);
            for this_cause in e.chain() {
                error!("{}", this_cause);
            }

            return;
        }
    };

    let end_game = clockwork.main_loop();

    match end_game {
        Ok(()) => {
            info!("🎉  Exiting cleanly and gracefully 🥂");
        }
        Err(e) => {
            error!("Runtime Error: {}", e);
            for this_cause in e.chain() {
                error!("{}", this_cause);
            }
        }
    };
}
