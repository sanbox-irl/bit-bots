use std::fmt;
use uuid::Uuid;

macro_rules! create_guarded_uuid {
    ($this_val:ident) => {
        #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Copy, Clone, Default)]
        pub struct $this_val(Uuid);

        impl fmt::Display for $this_val {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl $this_val {
            /// Creates a new and blank Id based on Uuid::v4.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Creates a new Id with the provided Uuid.
            pub fn with_id(id: Uuid) -> Self {
                Self(id)
            }

            /// Gives access to the inner ID. Try to not use this one too much!
            pub fn inner(&self) -> Uuid {
                self.0
            }
        }
    };
}

create_guarded_uuid!(SerializationId);
create_guarded_uuid!(PrefabId);
