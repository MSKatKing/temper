use std::cell::RefCell;
use std::sync::{Arc, Weak};
use crate::location::ResourceLocation;
use crate::resource::Resource;

pub struct ResourceHolder<T: Resource> {
    location: ResourceLocation<'static, T>,
    reference: RefCell<Weak<T>>,
}

impl<T: Resource> ResourceHolder<T> {
    /// Attempts to grab a new ResourceHolder from the central resource manager.
    ///
    /// # Arguments
    ///  * `namespace`: the namespace that the resource is located in.
    ///  * `path`: the path that the resource is located at.
    ///
    /// # Returns
    ///  * `Some(self)`: the resource was successfully fetched.
    ///  * `None`: the resource could not be found.
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Option<Self> {
        let location = ResourceLocation::<T>::new(namespace, path);
        let reference = Weak::new(); // TODO: get actual reference

        Some(Self { location, reference: RefCell::new(reference) })
    }

    /// Creates a new holder without checking if it is valid.
    ///
    /// # Arguments
    ///  * `namespace`: the namespace that the resource is located in.
    ///  * `path`: the path that the resource is located at.
    pub const fn new_unchecked(namespace: &'static str, path: &'static str) -> Self {
        let location = ResourceLocation::<T>::new_ref(namespace, path);

        Self {
            location,

            // when this Weak ptr is upgraded it will always return None, so we check for validity
            // on the first load.
            reference: RefCell::new(Weak::new()),
        }
    }

    /// Attempts to grab a new ResourceHolder from the central resource manager at the default
    /// namespace.
    ///
    /// # Arguments
    ///  * `path`: the path that the resource is located at within the default namespace.
    ///
    /// # Returns
    ///  * `Some(self)`: the resource was successfully fetched.
    ///  * `None`: the resource could not be found.
    pub fn with_default_namespace(path: impl Into<String>) -> Option<Self> {
        let location = ResourceLocation::<T>::with_default_namespace(path);
        let reference = Weak::new(); // TODO: get actual reference

        Some(Self { location, reference: RefCell::new(reference) })
    }

    /// Creates a new holder to a resource in the default namespace without checking if it is valid.
    ///
    /// # Arguments
    ///  * `path`: the path that the resource is located at in the default namespace.
    pub const fn with_default_namespace_unchecked(path: &'static str) -> Self {
        let location = ResourceLocation::<T>::with_default_namespace_ref(path);

        Self {
            location,

            // when this Weak ptr is upgraded it will always return None, so we check for validity
            // on the first load.
            reference: RefCell::new(Weak::new()),
        }
    }

    /// Attempts to grab the data referenced by this holder. This method may still fail even if
    /// [ResourceHolder::new] succeeded because resources may have been reloaded.
    ///
    /// # Returns
    ///  * `Some(resource)`: the resource that this holder references.
    ///  * `None`: the reference could not be found.
    pub fn get(&self) -> Option<Arc<T>> {
        if let Some(resource) = self.reference.borrow().upgrade() {
            return Some(resource);
        }

        todo!("try get new resource")
    }
}
