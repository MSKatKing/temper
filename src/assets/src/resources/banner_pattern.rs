use crate::registry::{IndexedRegistry, Registry};
use crate::resource::{IndexedResource, Resource};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::Deref;
use crate::resources::gather_indexed;

#[derive(Deserialize, Debug, Clone)]
pub struct BannerPattern {
    pub asset_id: Cow<'static, str>,
    pub translation_key: Cow<'static, str>,
}

include!(concat!(env!("OUT_DIR"), "/resources/banner_patterns.rs"));

lazy_static!(
    static ref BANNER_PATTERN_REGISTRY: BTreeMap<String, BTreeMap<String, BannerPattern>> = gather_indexed(
        &BannerPattern::PATTERNS
    );
);

impl Resource for BannerPattern {
    const ROOT_DIR: &'static str = "banner_pattern";

    fn registry() -> &'static impl Registry<Self> {
        BANNER_PATTERN_REGISTRY.deref()
    }
}

impl IndexedResource for BannerPattern {
    fn indexed_registry() -> &'static impl IndexedRegistry<Self> {
        BANNER_PATTERN_REGISTRY.deref()
    }
}
