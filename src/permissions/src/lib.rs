use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use type_hash::TypeHash;

pub mod player;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, TypeHash)]
pub enum Permissions {
    ALL,

    StopServer,
    Teleport,
    Kill,
    Ban,
    Kick,
    Op,
    DeOp,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, TypeHash)]
pub enum Access {
    Allow,
    Deny,
}

pub type PermissionSet = HashMap<Permissions, Access>;
