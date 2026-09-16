use std::borrow::Cow;
use std::marker::PhantomData;
use crate::resource::Resource;

#[derive(Debug)]
pub struct ResourceLocation<'a, T: Resource> {
    pub namespace: Cow<'a, str>,
    pub path: Cow<'a, str>,
    __inner: PhantomData<T>,
}

impl<T: Resource> ResourceLocation<'_, T> {
    pub(crate) const DEFAULT_NAMESPACE: &'static str = "minecraft";
    
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> ResourceLocation<'static, T> {
        ResourceLocation::<'static, T> {
            namespace: Cow::Owned(namespace.into()),
            path: Cow::Owned(path.into()),
            __inner: PhantomData,
        }
    }
    
    pub const fn new_ref<'a>(namespace: &'a str, path: &'a str) -> ResourceLocation<'a, T> {
        ResourceLocation::<'a, T> {
            namespace: Cow::Borrowed(namespace),
            path: Cow::Borrowed(path),
            __inner: PhantomData,
        }
    }
    
    pub fn with_default_namespace(path: impl Into<String>) -> Self {
        Self::new(Self::DEFAULT_NAMESPACE, path)
    }
    
    pub const fn with_default_namespace_ref(path: &str) -> ResourceLocation<T> {
        Self::new_ref(Self::DEFAULT_NAMESPACE, path)
    }
}

impl<T: Resource> PartialEq for ResourceLocation<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.namespace.eq(other.namespace.as_ref())
            && self.path.eq(other.path.as_ref())
    }
}
