use crate::registry::{IndexedRegistry, Registry};

/// A trait to be implemented on types that represent specific assets located in datapacks.
pub(crate) trait Resource: Clone + Send + Sync + 'static {
    /// The root path where assets of this type are located. This should be relative to the
    /// namespace root. For example, the root path for the type that represents achievements would
    /// simply be "achievements" (as achievements are located at "data/{namespace}/achievements".
    const ROOT_DIR: &'static str;
    
    /// Fetch the registry for this resource from the central registry singleton.
    fn registry() -> &'static impl Registry<Self>;
}

/// An optional extension to the [Resource] trait that requires that the backing registry maintains
/// the order of resources within it.
pub(crate) trait IndexedResource: Resource {
    /// Fetch the registry for this resource from the central registry singleton. This registry
    /// must preserve the order of resources within it. The value returned should be the same as
    /// [Resource::registry]. This method is just to restrict the output type to any type that
    /// implements [IndexedRegistry].
    fn indexed_registry() -> &'static impl IndexedRegistry<Self>;
}
