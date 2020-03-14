#![warn(elided_lifetimes_in_paths)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate bit_bots_derive;
#[macro_use]
extern crate anyhow;

mod entities {
    pub use super::*;

    mod component_list;
    mod generational_index;
    mod generational_index_array;
    mod generational_index_value;
    use generational_index::*;

    pub use generational_index_value::GenerationalIndexValue;
    pub type Entity = generational_index::GenerationalIndex;
    pub type EntityAllocator = GenerationalIndexAllocator;
    pub type ComponentList<T> = generational_index_array::GenerationalIndexArray<super::Component<T>>;
}

mod components_singleton {
    use super::*;

    mod camera;
    mod markers;
    mod rendering_utility;
    mod singleton_component;

    pub use camera::{Camera, CameraMode};
    pub use markers::Marker;
    pub use rendering_utility::{BasicTextures, RenderingUtility};
    pub use singleton_component::{SingletonBounds, SingletonComponent};
}

mod components {
    pub use super::*;

    mod component;
    // pub mod component_serialization;
    mod component_utils;
    mod conversant_npc;
    mod draw_rectangle;
    mod follow;
    mod name;
    pub mod physics_components;
    mod player;
    mod prefab_marker;
    mod scene_switcher;
    mod sound_source;
    mod sprite;
    mod text_source;
    // pub mod tilemap;
    mod transform;
    mod velocity;

    pub use {
        component::*,
        component_utils::{
            bounding_circle::BoundingCircle, component_traits::*, draw_layer::*, imgui_component_utils,
            Approach, DrawCommand, EditingMode, GameWorldDrawCommands, ImGuiDrawCommands, PositionalRect,
            SerializableEntityReference, SerializablePrefabReference, Tile,
        },
        conversant_npc::*,
        draw_rectangle::*,
        follow::*,
        name::Name,
        player::Player,
        prefab_marker::*,
        scene_switcher::SceneSwitcher,
        sound_source::SoundSource,
        sprite::Sprite,
        text_source::TextSource,
        transform::Transform,
        velocity::Velocity,
    };
}

mod hardware_interfaces {
    pub use super::*;

    mod hardware_interface;
    mod input;
    mod renderer;
    mod sound_player;
    mod window;

    pub use input::{Input, KeyboardInput, MouseButton, MouseInput};
    pub use renderer::{
        BufferBundle, DrawingError, ImguiPushConstants, LoadedImage, PipelineBundle, RendererComponent,
        RendererCreationError, StandardPushConstants, StandardQuad, StandardQuadFactory, StandardTexture,
        TextureDescription, Vertex, VertexIndexPairBufferBundle,
    };

    pub use hardware_interface::HardwareInterface;
    // pub use sound_player::SoundPlayer;
    pub use window::Window;
}

mod resources {
    use super::*;

    pub mod fonts;
    pub mod game_config;
    mod prefab;
    mod resources_database;
    mod sound_resource;
    pub mod sprite_resources;
    pub mod tile_resources;

    pub use prefab::*;
    pub use resources_database::ResourcesDatabase;
    pub use sound_resource::SoundResource;
}

mod serialization {
    pub use super::*;

    mod fragmented_data;
    pub mod serialization_util;
    mod serialized_entity;

    #[cfg(debug_assertions)]
    pub mod update_serialization;

    pub use fragmented_data::FragmentedData;
    pub use serialized_entity::*;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum SerializationDelta {
        Unchanged,
        Updated,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[must_use]
    pub struct PostDeserializationRequired;

    impl PostDeserializationRequired {
        pub fn fold_in(&self, _: PostDeserializationRequired) {}
    }
}

mod tick_structs {
    pub use super::*;

    // mod discord_rpc;
    mod clipboard_support;
    mod imgui;
    mod time_keeper;

    pub use self::imgui::{ImGui, ImGuiFlags, ImGuiMetaData, UiHandler};
    pub use clipboard_support::ClipboardSupport;
    // pub use discord_rpc::DiscordSDK;
    pub use time_keeper::TimeKeeper;
}

mod systems {
    pub use super::*;

    pub mod cross_cutting_system;
    pub mod follow_system;
    pub mod imgui_system;
    pub mod input_system;
    pub mod physics_system;
    pub mod player_system;
    pub mod prefab_system;
    pub mod renderer_system;
    pub mod scene_graph_system;
    pub mod scene_system;
    pub mod singleton_systems;
    pub mod sound_system;
    pub mod sprite_system;
    pub mod tilemap_system;
}

mod clockwork {
    use super::*;
    mod action_map;
    mod clockwork;
    mod component_database;
    mod ecs;
    mod scene;
    mod scene_data;
    mod singleton_database;

    pub use action_map::*;
    pub use clockwork::Clockwork;
    pub use component_database::{ComponentDatabase, NonInspectableEntities};
    pub use ecs::Ecs;
    pub use scene::*;
    pub use scene_data::*;
    pub use singleton_database::{AssociatedEntityMap, SingletonDatabase};
}

mod utilities {
    use super::imgui_system;

    mod axis;
    mod cached_bool;
    pub mod cardinals;
    mod color;
    mod guarded_rw_lock;
    mod guarded_uuids;
    pub mod math;
    mod rect;
    mod vec;

    pub mod number_util;
    pub use axis::Axis;
    pub use cached_bool::CachedBool;
    pub use color::Color;
    pub use guarded_rw_lock::*;
    pub use guarded_uuids::*;
    pub use rect::Rect;
    pub use vec::{Vec2, Vec2Int};
}

pub mod scene_graph {
    #[macro_use]
    mod relations;

    mod graph;
    mod graph_id;
    mod node;
    mod node_error;
    mod siblings_range;
    mod traverse;

    pub use node_error::*;

    use super::{Entity, SerializationId};

    pub type SceneGraph = graph::Graph<Entity>;
    pub type Node = node::GraphNode<Entity>;
    pub type NodeId = graph_id::GraphId<Entity>;

    pub type SerializedSceneGraph = graph::Graph<SerializationId>;
    pub type SerializedNode = node::GraphNode<SerializationId>;
    pub type SerializedNodeId = graph_id::GraphId<SerializationId>;

    impl SceneGraph {
        pub fn pretty_print(&self, names: &super::ComponentList<super::Name>) {
            self.print_tree(|node| println!("{}", super::Name::get_name_quick(names, node.inner())));
        }
    }
}

pub use clockwork::*;
pub use components::*;
pub use components_singleton::*;
pub use entities::*;
pub use hardware_interfaces::*;
pub use resources::*;
pub use serialization::*;
pub use systems::*;
pub use tick_structs::*;
pub use utilities::*;

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
