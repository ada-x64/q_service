use bevy_derive::{Deref, DerefMut};
use bevy_ecs::resource::Resource;
use bevy_platform::collections::HashMap;

use crate::prelude::*;
use std::{fmt::Debug, hash::Hash};

/// Used to specify where and how the service failed.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceError {
    /// The service failed all by itself!
    #[error("{0}")]
    Own(String),
    // Not boxing here because IsServiceError is not dyn compatible.
    /// A dependency failed, propogating to this service.
    #[error("Dependency {0} failed with error:\n{1}")]
    Dependency(String, String),
}

// #[derive(Debug, States, Deref)]
// pub struct ServiceStates<T: Service>(#[deref] ServiceState, PhantomData<T>);
// impl<T: Service> ServiceStates<T> {
//     pub fn new(status: ServiceState) -> Self {
//         Self(status, PhantomData)
//     }
// }
// impl<T> Clone for ServiceStates<T>
// where
//     T: Service,
// {
//     fn clone(&self) -> Self {
//         Self(self.0.clone(), self.1)
//     }
// }
// impl<T> PartialEq for ServiceStates<T>
// where
//     T: Service,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0 && self.1 == other.1
//     }
// }
// impl<T> Eq for ServiceStates<T> where T: Service {}
// impl<T> Hash for ServiceStates<T>
// where
//     T: Service,
// {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.0.hash(state);
//         self.1.hash(state);
//     }
// }

/// Tracks the current state of the service.
/// In order to react to changes, use [events](crate::lifecycle::events) or
/// [service hooks](crate::lifecycle::hooks).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ServiceStatus {
    /// The service is currently down.
    Down(DownReason),
    /// The service is asychronously deinitializing.
    Deinit(DownReason),
    /// The service is asychronously initializing.
    Init,
    /// The service is up and running.
    Up,
}
impl ServiceStatus {
    /// Self::Down(DownReason::SpunDown)
    pub fn down() -> Self {
        Self::Down(DownReason::SpunDown)
    }
    /// Self::Deinit(DownReason::SpunDown)
    pub fn deinit() -> Self {
        Self::Deinit(DownReason::SpunDown)
    }
    /// Self::Down(DownReason::Failed(reason))
    pub fn failed(reason: ServiceError) -> Self {
        Self::Down(DownReason::Failed(reason))
    }
    /// Self::Deinit(DownReason::Failed(reason))
    pub fn failing(reason: ServiceError) -> Self {
        Self::Deinit(DownReason::Failed(reason))
    }
    /// Self::Down(DownReason::Uninitialized)
    pub fn uninit() -> Self {
        Self::Down(DownReason::Uninitialized)
    }
}
impl Default for ServiceStatus {
    fn default() -> Self {
        Self::Down(DownReason::Uninitialized)
    }
}
impl ServiceStatus {
    #[allow(missing_docs)]
    pub fn is_down(&self) -> bool {
        matches!(self, ServiceStatus::Down(_))
    }
    #[allow(missing_docs)]
    pub fn is_initializing(&self) -> bool {
        matches!(self, ServiceStatus::Init)
    }
    #[allow(missing_docs)]
    pub fn is_up(&self) -> bool {
        matches!(self, ServiceStatus::Up)
    }
    #[allow(missing_docs)]
    pub fn is_failed(&self) -> bool {
        matches!(self, ServiceStatus::Down(DownReason::Failed(_)))
    }
    #[allow(missing_docs)]
    pub fn is_failing(&self) -> bool {
        matches!(self, ServiceStatus::Deinit(DownReason::Failed(_)))
    }
    #[allow(missing_docs)]
    pub fn is_deinitializing(&self) -> bool {
        matches!(self, ServiceStatus::Deinit(_))
    }
}
/// Describes the reason the service is currently down.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DownReason {
    /// The service hasn't yet been initialized.
    Uninitialized,
    /// At some point, this service failed. Contains the error, which might be from a dependency.
    /// See [ServiceError] for more details.
    Failed(ServiceError),
    /// The service succesfully spun down.
    SpunDown,
}
impl DownReason {
    /// The service itself failed. Distinct from [DownReason::dep_failure()]
    pub fn failed(err: impl ToString) -> Self {
        Self::Failed(ServiceError::Own(err.to_string()))
    }
    /// One of the service's dependencies failed. Distint from [DownReason::failed()].
    pub fn dep_failure<Dependency: Service>(err: impl ToString) -> Self {
        Self::Failed(ServiceError::Dependency(
            Dependency::name().to_string(),
            err.to_string(),
        ))
    }
}

/// The main storage mechanism for services.
///
/// Services and their dependencies are stored as nodes within a dependency
/// graph. All associated data is stored here for efficiency's sake.
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct GraphDataCache(HashMap<NodeId, GraphData>);
#[allow(missing_docs, reason = "obvious")]
impl GraphDataCache {
    pub fn get_service(&self, id: NodeId) -> Option<&ServiceData> {
        self.get(&id).and_then(|dep| dep.as_service())
    }
    pub fn get_service_mut(&mut self, id: NodeId) -> Option<&mut ServiceData> {
        self.get_mut(&id).and_then(|dep| dep.as_service_mut())
    }

    pub fn get_resource(&self, id: NodeId) -> Option<&ResourceData> {
        self.get(&id).and_then(|dep| dep.as_resource())
    }
    pub fn get_resource_mut(&mut self, id: NodeId) -> Option<&mut ResourceData> {
        self.get_mut(&id).and_then(|dep| dep.as_resource_mut())
    }

    pub fn get_asset(&self, id: NodeId) -> Option<&AssetData> {
        self.get(&id).and_then(|dep| dep.as_asset())
    }
    pub fn get_asset_mut(&mut self, id: NodeId) -> Option<&mut AssetData> {
        self.get_mut(&id).and_then(|dep| dep.as_asset_mut())
    }
}

/// Gets the name of a type as a string.
/// Truncates up to the last colon.
pub fn name_from_type<T>() -> String {
    // of form "some::path::to::service_impl::MyService"
    let mut base = std::any::type_name::<T>();
    let last_colon = base.rfind(':');
    if let Some(idx) = last_colon {
        base = base.split_at(idx + 1).1;
    }
    base.to_string()
}
