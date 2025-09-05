use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A type map for storing extensions
#[derive(Debug, Clone, Default)]
pub struct Extensions {
    map: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl Extensions {
    /// Create a new empty Extensions map
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a value into the extensions map
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        let mut map = self.map.write().unwrap();
        map.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get a reference to a value from the extensions map
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>>
    where
        T: Clone,
    {
        let map = self.map.read().unwrap();
        map.get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .map(|value| Arc::new(value.clone()))
    }

    /// Remove a value from the extensions map
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        let mut map = self.map.write().unwrap();
        map.remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok())
            .map(|boxed| *boxed)
    }

    /// Check if the extensions map contains a value of type T
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        let map = self.map.read().unwrap();
        map.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of extensions
    pub fn len(&self) -> usize {
        let map = self.map.read().unwrap();
        map.len()
    }

    /// Check if the extensions map is empty
    pub fn is_empty(&self) -> bool {
        let map = self.map.read().unwrap();
        map.is_empty()
    }

    /// Clear all extensions
    pub fn clear(&mut self) {
        let mut map = self.map.write().unwrap();
        map.clear();
    }
}

/// A wrapper for extension values that can be extracted in handlers
#[derive(Debug)]
pub struct Extension<T>(pub T);

impl<T> Extension<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Extension<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Extension<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
