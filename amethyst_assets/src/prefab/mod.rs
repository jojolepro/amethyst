mod importers;
pub use importers::PrefabImporter;

mod assets;
pub use assets::{Prefab, RawPrefab};

mod component_registry;
pub use component_registry::{ComponentRegistry, ComponentRegistryBuilder};
pub use legion_prefab::{register_component_type, ComponentRegistration};
