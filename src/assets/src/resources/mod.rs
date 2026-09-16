mod banner_pattern;

use std::collections::BTreeMap;
use serde::de::DeserializeOwned;
pub use banner_pattern::BannerPattern;
use crate::location::ResourceLocation;
use crate::resource::IndexedResource;

fn gather_indexed<T: IndexedResource + DeserializeOwned + Clone>(
    default_data: &[(&'static str, T)]
) -> BTreeMap<String, BTreeMap<String, T>> {
    let mut default_namespace = BTreeMap::new();

    for (path, data) in default_data {
        default_namespace.insert(path.to_string(), data.clone());
    }

    [(ResourceLocation::<T>::DEFAULT_NAMESPACE.to_string(), default_namespace)]
        .into_iter()
        .collect()
}
