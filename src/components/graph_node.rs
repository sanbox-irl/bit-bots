use super::{
    component_utils::RawComponent, imgui_system, ComponentBounds, ComponentData, ComponentList, Entity,
    InspectorParameters, SerializableEntityReference, SerializationMarker, Transform,
};

#[derive(Debug, Clone,SerializableComponent, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct GraphNode {
    pub children: Option<Vec<SerializableEntityReference>>,
}

impl GraphNode {
    #[allow(dead_code)]
    pub fn specific_entity_inspector(
        &mut self,
        entity_id: Entity,
        ip: InspectorParameters<'_, '_>,
        serializations: &ComponentList<SerializationMarker>,
        transforms: &mut ComponentList<Transform>,
    ) {
        if let Some(our_children) = &self.children {
            for this_child in our_children {
                if let Some(this_child_target) = this_child.target {
                    ip.ui
                        .bullet_text(&imgui::im_str!("{}##{}", this_child_target, ip.uid));
                } else {
                    ip.ui.bullet_text(imgui::im_str!("Blank Child!"));
                }
            }
        } else {
            ip.ui.text("None");
        }

        if let Some(new_child) =
            imgui_system::select_entity("Add Child", ip.uid, ip.ui, ip.entities, ip.entity_names)
        {
            self.add_child(Some(entity_id), new_child, transforms, serializations);
        }
    }

    /// Use this to add a child to a GraphNode. It handles calling all subordinate
    /// functions in the Transform and all other classes. It also makes a RawComponent
    /// for you, so RC never needs to be seen outside this sub-folder.
    ///
    /// Use this for most child operations.
    /// Use `add_child_directly` if you want to use the transform directly:
    /// `add_child_directly`should be used for the RootNode or when iterating over transforms;
    /// otherwise, prefer `add_child`
    pub fn add_child(
        &mut self,
        my_entity_id: Option<Entity>,
        new_child: Entity,
        transforms: &mut ComponentList<Transform>,
        serializations: &ComponentList<SerializationMarker>,
    ) {
        // If they have a transform, work on it.
        // It is possible to have a child without a transform, if that child is
        // essentially just a folder.
        if let Some(trans) = transforms.get_mut(&new_child) {
            let id = trans.entity_id();
            trans.inner_mut().set_new_parent(
                id,
                RawComponent {
                    entity: my_entity_id,
                    graph_node: &mut *self,
                },
            );
        }

        // Make the reference to the new_child...
        let new_child_reference =
            SerializableEntityReference::from_entity_id(Some(new_child), serializations);

        match &mut self.children {
            Some(children) => children.push(new_child_reference),
            None => self.children = Some(vec![new_child_reference]),
        }
    }

    /// Use this function to add a child to a GraphNode. Prefer to use `add_child` over
    /// `add_child_directly` when possible. This function is made available for iterations
    /// especially for the Root Node.
    pub fn add_child_directly(
        &mut self,
        my_entity_id: Option<Entity>,
        mut transform_c: ComponentData<'_, Transform>,
        serializations: &ComponentList<SerializationMarker>,
    ) {
        let id = transform_c.entity_id();
        transform_c.inner_mut().set_new_parent(
            id,
            RawComponent {
                entity: my_entity_id,
                graph_node: &mut *self,
            },
        );

        assert_ne!(my_entity_id, Some(id));

        // Make the reference to the new_child...
        let new_child_reference = SerializableEntityReference::from_entity_id(Some(id), serializations);

        match &mut self.children {
            Some(children) => children.push(new_child_reference),
            None => self.children = Some(vec![new_child_reference]),
        }
    }
}

impl ComponentBounds for GraphNode {
    fn entity_inspector(&mut self, _ip: InspectorParameters<'_, '_>) {
        unimplemented!();
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .graph_node
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.graph_node = Some({
            let mut clone: super::GraphNode = self.clone();
            if let Some(children) = clone.children.as_mut() {
                for child in children.iter_mut() {
                    child.entity_id_to_serialized_refs(&serialization_markers);
                }
            }

            super::SerializedComponent { inner: clone, active }
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.graph_node = None;
    }

    fn post_deserialization(
        &mut self,
        _: super::Entity,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.serialized_refs_to_entity_id(&serialization_markers);
            }
        }
    }
}
