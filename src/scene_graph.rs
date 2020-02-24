use super::{
    imgui_component_utils::NameInspectorParameters, ComponentData, ComponentDatabase, ComponentList, Entity,
    GraphNode, Name, PrefabMarker, ResourcesDatabase, SerializationMarker, SerializedEntity,
    SingletonDatabase, Transform, Vec2,
};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref ROOT_NODES: Mutex<GraphNode> = Mutex::new(GraphNode {
        children: Some(vec![])
    });
}

pub fn add_to_scene_graph<'a>(
    transform_c: impl Into<ComponentData<'a, Transform>>,
    serializations: &ComponentList<SerializationMarker>,
) {
    ROOT_NODES
        .lock()
        .unwrap()
        .add_child_directly(None, transform_c.into(), serializations);
}

pub fn clear_root() {
    if let Some(children) = &mut ROOT_NODES.lock().unwrap().children {
        children.clear();
    }
}

pub fn walk_graph(transforms: &mut ComponentList<Transform>, nodes: &ComponentList<GraphNode>) {
    let root_nodes = ROOT_NODES.lock().unwrap();

    if let Some(root_nodes) = &root_nodes.children {
        for secondary_node in root_nodes {
            if let Some(target) = &secondary_node.target {
                walk_node(target, transforms, nodes, Vec2::ZERO);
            }
        }
    }
}

fn walk_node(
    entity: &Entity,
    transforms: &mut ComponentList<Transform>,
    nodes: &ComponentList<GraphNode>,
    last_world_position: Vec2,
) {
    let new_world_position = if let Some(transform) = transforms.get_mut(entity) {
        transform.inner_mut().update_world_position(last_world_position)
    } else {
        last_world_position
    };

    if let Some(this_node) = nodes.get(entity) {
        if let Some(children) = &this_node.inner().children {
            for child in children {
                if let Some(target) = &child.target {
                    walk_node(target, transforms, nodes, new_world_position);
                }
            }
        }
    }
}

type GraphInspectorLambda<'a> = &'a mut dyn FnMut(
    &Entity,
    &mut ComponentList<Name>,
    &mut ComponentList<SerializationMarker>,
    Option<SerializedEntity>,
    &ComponentList<PrefabMarker>,
    NameInspectorParameters,
) -> bool;

pub fn walk_graph_inspect(
    component_database: &mut super::ComponentDatabase,
    singleton_database: &mut SingletonDatabase,
    resources: &ResourcesDatabase,
    f: GraphInspectorLambda<'_>,
) {
    let root_nodes = ROOT_NODES.lock().unwrap();

    if let Some(root_nodes) = &root_nodes.children {
        for secondary_node in root_nodes {
            if let Some(target) = &secondary_node.target {
                walk_node_inspect(target, component_database, singleton_database, resources, 0, f);
            }
        }
    }
}

fn walk_node_inspect(
    entity: &Entity,
    component_database: &mut ComponentDatabase,
    singleton_database: &mut SingletonDatabase,
    resources: &ResourcesDatabase,
    depth: usize,
    f: GraphInspectorLambda<'_>,
) {
    // Unwrap our parts:
    let current_se: Option<SerializedEntity> =
        if let Some(se) = component_database.serialization_markers.get(entity) {
            SerializedEntity::new(
                entity,
                se.inner().id,
                component_database,
                singleton_database,
                resources,
            )
        } else {
            None
        };

    let mut show_children = true;
    let has_children: bool = component_database
        .graph_nodes
        .get(entity)
        .map(|n| {
            if let Some(children) = &n.inner().children {
                children.len() > 0
            } else {
                false
            }
        })
        .unwrap_or_default();

    let has_transform = component_database.transforms.contains(entity);

    if has_transform {
        show_children = f(
            entity,
            &mut component_database.names,
            &mut component_database.serialization_markers,
            current_se,
            &component_database.prefab_markers,
            NameInspectorParameters {
                depth,
                has_children,
                serialization_status: Default::default(),
                prefab_status: Default::default(),
                being_inspected: Default::default(),
            },
        );
    }

    if show_children {
        // We're forced to break Rust borrowing rules here again, because we're a bad bitch.
        let graph_nodes: *const ComponentList<GraphNode> = &component_database.graph_nodes;

        if let Some(this_node) = unsafe { &*graph_nodes }.get(entity) {
            if let Some(children) = &this_node.inner().children {
                for child in children {
                    if let Some(target) = &child.target {
                        walk_node_inspect(
                            target,
                            component_database,
                            singleton_database,
                            resources,
                            depth + 1,
                            f,
                        );
                    }
                }
            }
        }
    }
}
