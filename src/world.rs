use bevy_ecs::world::{Mut, World};

use crate::prelude::*;

/// Extension trait for the World.
pub trait ServiceWorldExt {
    /// Gets a service by its handle.
    /// For a non-panicking alternative, see [ServiceWorldExt::get_service]
    /// # Panics
    /// Panics if the service is not registered.
    fn service<T: Service>(&self) -> &ServiceData;
    /// Mutably gets a service by its handle.
    /// For a non-panicking alternative, see [ServiceWorldExt::get_service_mut]
    /// # Panics
    /// Panics if the service is not registered.
    fn service_mut<'w, T: Service>(&'w mut self) -> Mut<'w, ServiceData>;

    /// Gets a service by its handle if it exists
    fn get_service<T: Service>(&self) -> Option<&ServiceData>;

    /// Mutably gets a service by its handle if it exists.
    fn get_service_mut<'w, T: Service>(&'w mut self) -> Option<Mut<'w, ServiceData>>;

    /// Gets a service by its ID.
    fn service_by_id(&self, id: NodeId) -> Option<&ServiceData>;

    /// Mutably gets a service by its ID.
    fn service_mut_by_id<'w>(&'w mut self, id: NodeId) -> Option<Mut<'w, ServiceData>>;

    /// Temporarily removes a service from the [GraphDataCache] in order to perform operations on it.
    /// # Panics
    /// Will panic if the service has not been registered.
    fn service_scope<T: Service, R>(
        &mut self,
        scope: impl FnMut(&mut Self, &mut ServiceData) -> R,
    ) -> R;
    /// See [ServiceWorldExt::service_scope]
    fn service_scope_by_id<R>(
        &mut self,
        id: NodeId,
        scope: impl FnMut(&mut Self, &mut ServiceData) -> R,
    ) -> R;
}

impl ServiceWorldExt for World {
    fn service<T: Service>(&self) -> &ServiceData {
        let id = NodeId::Service(self.resource_id::<T>().unwrap());
        self.resource::<GraphDataCache>().get_service(id).unwrap()
    }

    fn service_mut<'w, T: Service>(&'w mut self) -> Mut<'w, ServiceData> {
        let id = NodeId::Service(self.resource_id::<T>().unwrap());
        self.resource_mut::<GraphDataCache>()
            .map_unchanged(|cache| cache.get_service_mut(id).unwrap())
    }

    fn service_by_id(&self, id: NodeId) -> Option<&ServiceData> {
        self.get_resource::<GraphDataCache>()
            .and_then(|c| c.get_service(id))
    }
    fn service_mut_by_id<'w>(&'w mut self, id: NodeId) -> Option<Mut<'w, ServiceData>> {
        self.get_resource_mut::<GraphDataCache>()
            .map(|c| c.map_unchanged(|c| c.get_service_mut(id).unwrap()))
    }

    fn get_service<T: Service>(&self) -> Option<&ServiceData> {
        let id = self
            .service::<T>()
            .registered()
            .then_some(NodeId::Service(self.resource_id::<T>()?))?;
        self.get_resource::<GraphDataCache>()
            .and_then(|c| c.get_service(id))
    }

    fn get_service_mut<'w, T: Service>(&'w mut self) -> Option<Mut<'w, ServiceData>> {
        let id = self
            .service::<T>()
            .registered()
            .then_some(NodeId::Service(self.resource_id::<T>()?))?;
        self.get_resource_mut::<GraphDataCache>()
            .map(|cache| cache.map_unchanged(|cache| cache.get_service_mut(id).unwrap()))
    }

    fn service_scope<T: Service, R>(
        &mut self,
        scope: impl FnOnce(&mut Self, &mut ServiceData) -> R,
    ) -> R {
        let id = NodeId::Service(self.resource_id::<T>().unwrap());
        let mut service = self
            .resource_mut::<GraphDataCache>()
            .remove(&id)
            .unwrap_or_else(|| panic!("ServiceData for {} should be in the cache.", T::name()));
        let res = scope(self, service.as_service_mut().unwrap());
        self.resource_mut::<GraphDataCache>().insert(id, service);
        res
    }

    fn service_scope_by_id<R>(
        &mut self,
        id: NodeId,
        scope: impl FnOnce(&mut Self, &mut ServiceData) -> R,
    ) -> R {
        let mut service = self
            .resource_mut::<GraphDataCache>()
            .remove(&id)
            .unwrap_or_else(
                || panic!("ServiceData for service id {id:?} should be in the cache.",),
            );
        let res = scope(self, service.as_service_mut().unwrap());
        self.resource_mut::<GraphDataCache>().insert(id, service);
        res
    }
}
