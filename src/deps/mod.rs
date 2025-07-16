pub(crate) mod graph;

use crate::{deps::graph::DependencyGraph, prelude::*};
use bevy_ecs::prelude::*;
use std::any::TypeId;
use tracing::*;

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Meta information about the service.
pub struct ServiceDepInfo {
    /// TypeId is used as the key in the dependency graph.
    pub type_id: TypeId,
    /// The display name is used for friendly errors.
    pub display_name: String,
    /// Has this dependency been initialized?
    pub is_initialized: bool,
    /// Is this dependency a service? If you're implementing this, probably
    /// not.
    pub is_service: bool,
    // The resource id for the dependency.
    // pub resource_id: ComponentId,
}

#[allow(missing_docs)]
/// Initialization error for dependencies.
#[derive(thiserror::Error, Debug)]
pub enum DepInitErr {
    #[error("Service {0} failed to initialize with error {1}")]
    Service(String, String),
}

/// Marks an item as a service dependency. While this is usually for other
/// Services, it can be used for any arbitrary type.
pub trait IsServiceDep: std::fmt::Debug + Send + Sync {
    /// Although this takes an exclusive borrow, please do not mutate world
    /// here. This is necessary for resource scoping.
    fn info(&self, world: &mut World) -> ServiceDepInfo;
    /// Initialize the dependency and update the dependency graph.
    fn initialize(&mut self, world: &mut World) -> Result<(), DepInitErr>;
}
impl<T, D, E> IsServiceDep for Service<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn info(&self, _world: &mut World) -> ServiceDepInfo {
        ServiceDepInfo {
            type_id: TypeId::of::<Self>(),
            display_name: ServiceHandle::from_service(self).to_string(),
            is_initialized: self.initialized,
            is_service: true,
            // resource_id: world.resource_id::<Self>().unwrap(),
        }
    }
    fn initialize(&mut self, world: &mut World) -> Result<(), DepInitErr> {
        let info = self.info(world);
        debug!("Initializing dependency: {info:#?}");
        let res = self.on_init(world).map_err(|e| {
            DepInitErr::Service(info.display_name.clone(), e.to_string())
        });

        // Update graph
        if world.get_resource::<DependencyGraph>().is_none() {
            world.init_resource::<DependencyGraph>();
        }
        world.resource_scope(|world, mut graph: Mut<DependencyGraph>| {
            graph.register_service(
                ServiceHandle::<T, D, E>::const_default(),
                self.deps.iter().map(|d| d.info(world)).collect(),
            ).expect("Failed to add dependencies in service spec!\n.. Spec = {spec:#?}")
        });

        world
            .resource_mut::<DependencyGraph>()
            .services
            .entry(info.type_id)
            .insert(info);
        res
    }
}
impl<T, D, E> IsServiceDep for ServiceHandle<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn info(&self, world: &mut World) -> ServiceDepInfo {
        world.resource_scope(|world, service: Mut<Service<T, D, E>>| {
            service.info(world)
        })
    }
    fn initialize(&mut self, world: &mut World) -> Result<(), DepInitErr> {
        world.resource_scope(|world, mut service: Mut<Service<T, D, E>>| {
            service.initialize(world)
        })
    }
}
