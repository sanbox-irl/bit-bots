use std::fmt;
use uuid::Uuid;

macro_rules! create_guarded_uuid {
    ($this_val:ident) => {
        #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Copy, Clone, Default)]
        pub struct $this_val(pub Uuid);

        impl fmt::Display for $this_val {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl $this_val {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn with_id(id: Uuid) -> Self {
                Self(id)
            }

            pub fn inner(&self) -> Uuid {
                self.0
            }
        }
    };
}

create_guarded_uuid!(SerializationId);
create_guarded_uuid!(PrefabId);
