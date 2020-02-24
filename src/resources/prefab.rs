use super::SerializedEntity;
use std::collections::HashMap;
use uuid::Uuid;

/// Where the Key in the HashMap is the same as the MainID in the Prefab.
pub type PrefabMap = HashMap<Uuid, Prefab>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Prefab {
    root_id: Uuid,
    valid: bool,
    pub members: HashMap<Uuid, SerializedEntity>,
}

impl Clone for Prefab {
    fn clone(&self) -> Self {
        // Make a new ID
        let new_root_id = Uuid::new_v4();

        // Find our Old Main and give it the new ID:
        let mut members = HashMap::with_capacity(self.members.len());

        for (old_id, old_member) in self.members.iter() {
            let new_key = if old_id == &self.root_id {
                new_root_id
            } else {
                Uuid::new_v4()
            };

            let mut new_member = old_member.clone();
            new_member.id = new_key;

            members.insert(new_key, new_member);
        }

        Prefab {
            root_id: new_root_id,
            members,
            valid: true,
        }
    }
}

impl Prefab {
    /// Creates a new Prefab with only a single member, which will
    /// also be the RootEntity
    pub fn new(root_entity: SerializedEntity) -> Prefab {
        let root_id = root_entity.id;
        let members = maplit::hashmap! {
            root_id => root_entity
        };

        Prefab {
            root_id,
            members,
            valid: true,
        }
    }

    pub fn new_blank() -> Prefab {
        let root_id = Uuid::new_v4();
        let members = maplit::hashmap! {
            root_id => SerializedEntity {
                id: root_id,
                ..Default::default()
            }
        };

        Prefab {
            root_id,
            members,
            valid: true,
        }
    }

    pub fn root_entity(&self) -> &SerializedEntity {
        &self.members[&self.root_id]
    }

    pub fn root_entity_mut(&mut self) -> &mut SerializedEntity {
        self.members.get_mut(&self.root_id).unwrap()
    }

    pub fn root_id(&self) -> Uuid {
        self.root_id
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn log_to_console(&self) {
        println!("---Console Log for {}---", self.root_id);
        println!("{:#?}", self);
        println!("------------------------");
    }
}
