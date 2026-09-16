use std::collections::{BTreeMap, HashMap};
use crate::holder::ResourceHolder;
use crate::location::ResourceLocation;
use crate::resource::{IndexedResource, Resource};

/// A trait for types that store instances of a [Resource].
pub trait Registry<T: Resource> {
    fn try_get(&'static self, location: &ResourceLocation<T>) -> Option<&'static T>;
    fn iter_assets(&'static self) -> impl Iterator<Item = ResourceHolder<T>>;
}

/// A trait for types that store instances of a [Resource]. An additional requirement for
/// this trait is that it must be able to return a unique index for a given [ResourceLocation], and
/// that index must not change throughout the lifetime of the program.
pub trait IndexedRegistry<T: IndexedResource>: Registry<T> {
    fn try_get_index(&'static self, location: &ResourceLocation<T>) -> Option<usize>;
}

impl<T: Resource> Registry<T> for HashMap<String, HashMap<String, T>> {
    fn try_get(&'static self, location: &ResourceLocation<T>) -> Option<&'static T> {
        self.get(location.namespace.as_ref())
            .and_then(|path_map| path_map.get(location.path.as_ref()))
    }

    fn iter_assets(&'static self) -> impl Iterator<Item=ResourceHolder<T>> {
        self
            .iter()
            .map(|(namespace, path_map)|
                path_map
                    .iter()
                    .map(|(path, reference)|
                        ResourceHolder::<T> {
                            location: ResourceLocation::new_ref(namespace.as_ref(), path.as_ref()),
                            reference
                        }
                    )
            )
            .flatten()
    }
}

impl<T: Resource> Registry<T> for BTreeMap<String, BTreeMap<String, T>> {
    fn try_get(&'static self, location: &ResourceLocation<T>) -> Option<&'static T> {
        self.get(location.namespace.as_ref())
            .and_then(|path_map| path_map.get(location.path.as_ref()))
    }

    fn iter_assets(&'static self) -> impl Iterator<Item=ResourceHolder<T>> {
        self
            .iter()
            .map(|(namespace, path_map)|
                path_map
                    .iter()
                    .map(|(path, reference)|
                        ResourceHolder::<T> {
                            location: ResourceLocation::new_ref(namespace.as_ref(), path.as_ref()),
                            reference,
                        }
                    )
            )
            .flatten()
    }
}

impl<T: IndexedResource> IndexedRegistry<T> for BTreeMap<String, BTreeMap<String, T>> {
    fn try_get_index(&'static self, location: &ResourceLocation<T>) -> Option<usize> {
        self
            .iter_assets()
            .enumerate()
            .find(|(_, holder)|
                &holder.location == location
            )
            .map(|(i, _)| i)
    }
}
