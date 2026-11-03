//! Stable, typed document IDs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new time-ordered UUIDv7 ID.
            #[allow(clippy::new_without_default)]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Builds an ID from a fixed value, for deterministic tests and fixtures.
            pub const fn from_u128(value: u128) -> Self {
                Self(Uuid::from_u128(value))
            }

            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }
    )+};
}

define_id!(ProjectId, SceneId, LayerId, DrawingId, StrokeId);
