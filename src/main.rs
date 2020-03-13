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
mod entities;
mod hardware_interfaces;
mod physics;
mod resources;
mod scene;
pub mod scene_graph;
mod serialization;
mod systems;
mod tick_structs;
mod top_level;
mod utilities;

pub use action_map::ActionMap;
pub use clockwork::*;
pub use components::*;
pub use components_singleton::*;
pub use top_level::*;
pub use entities::*;
pub use hardware_interfaces::*;
pub use physics::*;
pub use resources::*;
pub use scene::*;
pub use serialization::*;
pub use systems::*;
pub use tick_structs::*;
pub use utilities::*;

// use scene_graph::SceneGraph;

fn main() {
    pretty_env_logger::init();

    // Update the database...
    #[cfg(debug_assertions)]
    {
        if update_serialization::UPDATE_COMPONENT_DATABASE {
            update_serialization::update_component_database()
                .expect_err("We failed to update serialization!");
        }
    }

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