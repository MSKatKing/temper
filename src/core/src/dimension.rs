use serde::{Deserialize, Serialize};
use type_hash::TypeHash;

/// Represents a dimension in the world. The first three variants are the standard Minecraft
/// dimensions, and the `Custom` variant can be used for modded dimensions or other custom implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TypeHash)]
pub enum Dimension {
    #[default]
    Overworld,
    Nether,
    End,
    Custom(u16),
}
