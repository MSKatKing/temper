use std::collections::BTreeMap;
use serde::de::DeserializeOwned;
use crate::location::ResourceLocation;
use crate::resource::IndexedResource;

mod resource;
mod holder;
mod location;
mod registry;
mod resources;

pub use resources::*;

pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
