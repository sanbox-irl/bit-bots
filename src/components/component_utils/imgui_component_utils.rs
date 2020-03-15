use super::{
    imgui_system, scene_graph::NodeId, Color, ComponentBounds, ComponentList, Entity, Name, PrefabId,
    PrefabMap, SceneMode, SerializationId, SerializedEntity,
};

const WARNING_ICON: char = '\u{f071}';
const SYNCED_ICON: char = '\u{f00c}';

pub struct InspectorParameters<'a, 'b> {
    pub ui: &'b imgui::Ui<'a>,
    pub entities: &'b [Entity],
    pub entity_names: &'b ComponentList<Name>,
    pub prefabs: &'b PrefabMap,
    pub uid: &'b str,
    pub is_open: bool,
}

#[derive(Debug)]
pub enum NameRequestedAction {
    ChangeName(String),
    ToggleInspect,

    Clone,
    Delete,

    PromoteToPrefab,
    UnpackPrefab,
    GoToPrefab,

    LogEntity,
    LogSerializedEntity,
    LogPrefab,

    EntitySerializationCommand(EntitySerializationCommandType),
    CreateEntityCommand(CreateEntityCommand),
}

#[derive(Debug, Copy, Clone)]
pub struct CreateEntityCommand {
    pub parent_id: Option<NodeId>,
    pub command_type: CreateEntityCommandType,
}

#[derive(Debug, Copy, Clone)]
pub enum CreateEntityCommandType {
    CreateBlank,
    CreatePrefab(PrefabId),
}

pub struct NameInspectorResult {
    pub show_children: bool,
    pub requested_action: Option<NameRequestedAction>,
}

impl Default for NameInspectorResult {
    fn default() -> Self {
        Self {
            show_children: true,
            requested_action: None,
        }
    }
}

impl NameInspectorResult {
    pub fn new() -> Self {
        let mut me: Self = Default::default();
        me.show_children = true;

        me
    }
}

#[derive(Debug)]
pub struct NameInspectorParameters<'a> {
    pub on_scene_graph: Option<NodeId>,
    pub has_children: bool,
    pub depth: usize,
    pub prefab_status: PrefabStatus,
    pub being_inspected: bool,
    pub scene_mode: SceneMode,
    pub serialization_status: SyncStatus,
    pub prefabs: &'a PrefabMap,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SyncStatus {
    Unsynced,
    Headless,
    OutofSync,
    Synced,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::Unsynced
    }
}

impl SyncStatus {
    pub fn new<T: ComponentBounds + Clone>(
        comp: &super::Component<T>,
        serialized_entity: Option<&SerializedEntity>,
        should_be_some: bool,
    ) -> SyncStatus {
        serialized_entity
            .map(|se| {
                if comp.is_serialized(se) {
                    SyncStatus::Synced
                } else {
                    SyncStatus::OutofSync
                }
            })
            .unwrap_or_else(|| {
                if should_be_some {
                    SyncStatus::Headless
                } else {
                    SyncStatus::Unsynced
                }
            })
    }

    pub fn is_synced_at_all(&self) -> bool {
        match self {
            SyncStatus::Unsynced => false,
            SyncStatus::Headless => false,
            SyncStatus::OutofSync => true,
            SyncStatus::Synced => true,
        }
    }

    pub fn imgui_color(&self, scene_mode: super::SceneMode) -> [f32; 4] {
        if scene_mode != super::SceneMode::Draft {
            Color::WHITE
        } else {
            match self {
                SyncStatus::Unsynced => Color::WHITE,
                SyncStatus::Headless => imgui_system::red_warning_color(),
                SyncStatus::OutofSync => imgui_system::prefab_light_blue_color(),
                SyncStatus::Synced => imgui_system::prefab_blue_color(),
            }
        }
        .into()
    }

    pub fn imgui_symbol(&self, scene_mode: super::SceneMode) -> char {
        if scene_mode != super::SceneMode::Draft {
            '\u{200B}'
        } else {
            match self {
                SyncStatus::Unsynced | SyncStatus::Headless | SyncStatus::OutofSync => WARNING_ICON,
                SyncStatus::Synced => SYNCED_ICON,
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ParentSyncStatus {
    pub serialized: SyncStatus,
    pub prefab: SyncStatus,
}

impl ParentSyncStatus {
    pub fn new<T: ComponentBounds + Clone>(
        comp: &super::Component<T>,
        serialized_entity: Option<&SerializedEntity>,
        prefab_entity: Option<&SerializedEntity>,
        should_have_prefab_entity: bool,
    ) -> ParentSyncStatus {
        ParentSyncStatus {
            serialized: serialized_entity.map_or(SyncStatus::Unsynced, |se| {
                if comp.is_serialized(se) {
                    SyncStatus::Synced
                } else {
                    SyncStatus::OutofSync
                }
            }),
            prefab: SyncStatus::new(comp, prefab_entity, should_have_prefab_entity),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrefabStatus {
    None,
    Prefab,
    PrefabInstanceRoot,
    PrefabInstanceSecondary,
}

impl PrefabStatus {
    pub fn is_prefab_inheritor(&self) -> bool {
        match self {
            PrefabStatus::PrefabInstanceRoot | PrefabStatus::PrefabInstanceSecondary => true,
            _ => false,
        }
    }
}

impl Default for PrefabStatus {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityListInformation {
    pub open: bool,
    pub color: Color,
    pub edit_name: Option<String>,
}

impl Default for EntityListInformation {
    fn default() -> Self {
        EntityListInformation {
            open: false,
            color: Color::WHITE.into(),
            edit_name: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ComponentInspectorListAction {
    Delete,
    RevertToParentPrefab,
    ComponentPostAction(ComponentSerializationCommandType),
    EntityPostAction(EntitySerializationCommandType),
}

#[derive(Debug, Clone)]
pub enum ComponentInspectorPostAction {
    ComponentCommands(ComponentSerializationCommand),
    EntityCommands(EntitySerializationCommand),
}

/// This serialization command is for **Component** serde
/// and **Component prefab** serde.
///
///  An example usage would be serializing a component `Sprite` on some Entity.
///
/// If we wanted to serialize an **Entity** which HAD a `Sprite`, we would use an
/// `EntitySerializationCommand`.
#[derive(Debug, Clone)]
pub struct ComponentSerializationCommand {
    /// This is the Entity ID of the target.
    pub entity: Entity,

    /// This is our Change to be Applied:
    /// - `Serialize` => New Component Data to Overwrite Old Data
    /// - `StopSerializing` => A Value::NULL
    /// - `Revert` => The old Serialized data to set on our live instance
    /// - `ApplyOverrideToParent` => This is the New Component Data to Overwrite Old Prefab Data
    /// - `RevertToParentPrefab` => This is the Old Prefab data to set on our live instance
    pub delta: serde_yaml::Value,

    /// This is, essentially, the name of our component as a YamlValue.
    /// For example, `Sprite` would be `sprite`. It will always be
    /// YamlValue::String(str)
    pub key: serde_yaml::Value,

    /// The Command to be Executed by the ImGui Serialization System
    pub command_type: ComponentSerializationCommandType,
}

#[derive(Debug, Copy, Clone)]
#[must_use]
pub enum ComponentSerializationCommandType {
    Serialize,
    StopSerializing,
    Revert,
    ApplyOverrideToParentPrefab,
    RevertToParentPrefab,
}

#[derive(Debug, Copy, Clone)]
pub struct EntitySerializationCommand {
    pub id: SerializationId,
    pub entity: Entity,
    pub command_type: EntitySerializationCommandType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntitySerializationCommandType {
    Overwrite,
    StopSerializing,
    Revert,
}
