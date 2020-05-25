use crate::Property;
use serde::Deserialize;
use std::{any::TypeId, collections::HashMap};

#[derive(Default)]
pub struct PropertyTypeRegistry {
    pub registrations: HashMap<String, PropertyTypeRegistration>,
}

impl PropertyTypeRegistry {
    pub fn register<T>(&mut self)
    where
        T: Property + for<'de> Deserialize<'de>,
    {
        let registration = PropertyTypeRegistration::of::<T>();
        self.registrations.insert(registration.short_name.to_string(), registration);
    }

    pub fn get(&self, type_name: &str) -> Option<&PropertyTypeRegistration> {
        self.registrations.get(type_name)
    }
}

#[derive(Clone)]
pub struct PropertyTypeRegistration {
    pub ty: TypeId,
    pub deserialize: fn(deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn Property>, erased_serde::Error>,
    pub short_name: &'static str,
}

impl PropertyTypeRegistration {
    pub fn of<T: Property + for<'de> Deserialize<'de>>() -> Self {
        let ty = TypeId::of::<T>();
        Self {
            ty,
            deserialize: |deserializer: &mut dyn erased_serde::Deserializer| {
                let property = <T as Deserialize>::deserialize(deserializer)?;
                Ok(Box::new(property))
            },
            short_name: std::any::type_name::<T>().split("::").last().unwrap(),
        }
    }
}