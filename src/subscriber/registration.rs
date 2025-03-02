//! Internal registration system for event handlers
use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Mutex, RwLock};
use once_cell::sync::Lazy;

use super::subscription::Subscription;
use crate::core::shared::SharedEventBus;

/// Function type for event handler registration methods
pub type HandlerRegistrationFn<T> = fn(&mut T, &SharedEventBus) -> Subscription;

/// Type-safe wrapper for static registry references
pub struct TypeRegistryRef<T: 'static + Send + Sync>(&'static TypeRegistry<T>);

// Implement needed traits
impl<T: 'static + Send + Sync> std::fmt::Debug for TypeRegistryRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypeRegistryRef").field(&"...").finish()
    }
}

// Explicitly mark as thread-safe
unsafe impl<T: 'static + Send + Sync> Send for TypeRegistryRef<T> {}
unsafe impl<T: 'static + Send + Sync> Sync for TypeRegistryRef<T> {}

/// Registry that stores handler methods for each component type
#[derive(Default)]
struct HandlerRegistry {
    registries: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

// Global registry for all handler types
static HANDLER_REGISTRY: Lazy<RwLock<HandlerRegistry>> = Lazy::new(|| {
    RwLock::new(HandlerRegistry::default())
});

/// Type-specific wrapper for handler registry access
pub struct TypeRegistry<T: 'static + Send + Sync> {
    _marker: PhantomData<T>,
    methods: Mutex<Vec<HandlerRegistrationFn<T>>>,
}

impl<T: 'static + Send + Sync> TypeRegistry<T> {
    /// Get or create the registry for this type
    pub fn get_or_create() -> &'static Self {
        let type_id = TypeId::of::<T>();
        
        if let Some(registry) = Self::find_existing_registry(type_id) {
            return registry;
        }
        
        Self::create_new_registry(type_id)
    }
    
    fn create_new_registry(type_id: TypeId) -> &'static Self {
        let mut registry_guard = HANDLER_REGISTRY.write().unwrap();
        
        // Double-check after acquiring write lock
        if let Some(existing) = Self::find_existing_registry(type_id) {
            return existing;
        }
        
        let new_registry = Box::new(TypeRegistry {
            _marker: PhantomData,
            methods: Mutex::new(Vec::new()),
        });
        
        // This leak is intentional - registries live for program duration
        let static_registry = Box::leak(new_registry);
        
        registry_guard.registries.insert(
            type_id, 
            Box::new(TypeRegistryRef(static_registry))
        );
        
        static_registry
    }

    fn find_existing_registry(type_id: TypeId) -> Option<&'static Self> {
        let registry = HANDLER_REGISTRY.read().unwrap();
        registry.registries.get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<TypeRegistryRef<T>>())
            .map(|wrapper| wrapper.0)
    }
    
    pub fn register_handler(&self, handler: HandlerRegistrationFn<T>) {
        if let Ok(mut methods) = self.methods.lock() {
            methods.push(handler);
        }
    }
    
    pub fn handlers(&self) -> Vec<HandlerRegistrationFn<T>> {
        match self.methods.lock() {
            Ok(methods) => methods.clone(),
            Err(_) => Vec::new(),
        }
    }
}

/// Register handler methods during initialization
pub fn register_handler_method<T: 'static + Send + Sync>(
    handler_method: HandlerRegistrationFn<T>
) {
    let registry = TypeRegistry::<T>::get_or_create();
    registry.register_handler(handler_method);
}

/// Get all handlers for a specific type
pub fn get_handlers_for_type<T: 'static + Send + Sync>() -> Vec<HandlerRegistrationFn<T>> {
    let registry = TypeRegistry::<T>::get_or_create();
    registry.handlers()
}