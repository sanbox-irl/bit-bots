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

    let mut scene_graph = SceneGraph::new();
    let n_1 = scene_graph.instantiate_node(Entity::debug_stub(0));
    let n_1_1 = scene_graph.instantiate_node(Entity::debug_stub(1));
    let n_1_2 = scene_graph.instantiate_node(Entity::debug_stub(2));
    n_1.append(n_1_1, &mut scene_graph);
    n_1.append(n_1_2, &mut scene_graph);

    let n_2 = scene_graph.instantiate_node(Entity::debug_stub(3));
    let n_2_1 = scene_graph.instantiate_node(Entity::debug_stub(4));
    n_2.append(n_2_1, &mut scene_graph);

    let n_2_1_1 = scene_graph.instantiate_node(Entity::debug_stub(5));
    let n_2_1_2 = scene_graph.instantiate_node(Entity::debug_stub(6));
    n_2_1.append(n_2_1_1, &mut scene_graph);
    n_2_1.append(n_2_1_2, &mut scene_graph);

    println!("Scene Graph:\n{}", scene_graph);

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
