//! State management

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
/// State management
pub struct State {
    data: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions").finish()
    }
}

impl State {
    /// Creates a new empty state
    pub fn new() -> Self {
        State {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inserts a value into the state
    pub fn insert<T: Send + Sync + 'static>(&self, value: T) {
        let mut data = self.data.write().unwrap();
        data.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Gets a value from the state
    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        let data = self.data.read().unwrap();
        data.get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.clone().downcast::<T>().ok())
    }
}
