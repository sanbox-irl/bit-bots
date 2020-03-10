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
pub mod scene_graph;
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
pub use serialization::*;
pub use systems::*;
pub use tick_structs::*;
pub use utilities::*;

// use scene_graph::SceneGraph;

fn main() {
    pretty_env_logger::init();

    // let mut scene_graph = SceneGraph::new();
    // let zero = scene_graph.instantiate_node(Entity::stub(0));
    // let one = scene_graph.instantiate_node(Entity::stub(1));
    // let two = scene_graph.instantiate_node(Entity::stub(2));
    // let one_one = scene_graph.instantiate_node(Entity::stub(3));
    // let two_one = scene_graph.instantiate_node(Entity::stub(4));
    // let two_two = scene_graph.instantiate_node(Entity::stub(5));

    // let scene_graph = &mut scene_graph;
    // two.append(two_one, scene_graph);
    // two.append(two_two, scene_graph);

    // one.append(one_one, scene_graph);

    // zero.append(one, scene_graph);
    // zero.append(two, scene_graph);

    // for descedants in zero.descendants(scene_graph).collect::<Vec<_>>().iter().rev() {
    //     println!("Descendant {}", scene_graph.get(*descedants).unwrap());
    // }

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
