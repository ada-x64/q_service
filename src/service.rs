//! The main service module. Defines the Service resource.

use crate::{
    deps::{DepInitErr, ServiceDep},
    prelude::*,
};
use bevy_ecs::{component::Tick, prelude::*};
use bevy_platform::prelude::*;
use tracing::*;

#[derive(Debug, Resource)]
/// Resource which represents a service.
pub struct Service<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    pub(crate) data: D,
    pub(crate) hooks: ServiceHooks<T, D, E>,
    pub(crate) state: ServiceState<E>,
    pub(crate) initialized: bool,
    pub(crate) deps: Vec<ServiceDep>,
    pub(crate) handle: ServiceHandle<T, D, E>,
    pub(crate) initialized_at: Option<Tick>,
    pub(crate) last_update: Option<Tick>,
}

impl<T, D, E> Service<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    /// Gets the default [ServiceSpec] for this service. Use the Spec to
    /// specify this service's behavior.
    pub fn default_spec() -> ServiceSpec<T, D, E> {
        ServiceSpec::default()
    }

    /// Gets the [ServiceHandle] for this service.
    pub fn handle() -> ServiceHandle<T, D, E> {
        ServiceHandle::const_default()
    }

    /// Gets the service's current state.
    /// In order to update this, use [commands](crate::lifecycle#commands) or [events](crate::lifecycle#events).
    pub fn state(&self) -> &ServiceState<E> {
        &self.state
    }

    /// Gets the service's data.
    /// In order to update this, use [commands](crate::lifecycle#commands) or [events](crate::lifecycle#events).
    pub fn data(&self) -> &D {
        &self.data
    }

    pub(crate) fn from_spec(spec: ServiceSpec<T, D, E>) -> Self {
        Self {
            data: spec.initial_data.unwrap_or_default(),
            state: ServiceState::default(),
            hooks: ServiceHooks {
                on_init: spec.on_init.unwrap_or_default(),
                on_enable: spec.on_enable.unwrap_or_default(),
                on_disable: spec.on_disable.unwrap_or_default(),
                on_failure: spec.on_failure.unwrap_or_default(),
                on_update: spec.on_update.unwrap_or_default(),
            },
            handle: ServiceHandle::const_default(),
            deps: spec.deps,
            initialized: false,
            initialized_at: None,
            last_update: None,
        }
    }

    pub(crate) fn init_as_dep(world: &mut World) -> Result<(), DepInitErr> {
        world.resource_scope(|world, mut this: Mut<Self>| {
            this.on_init(world)
                .map_err(|e| DepInitErr::Service(this.handle.to_string(), e.to_string()))
        })
    }

    /// Initializes the service. Depending on the result of the hook, it will
    /// then either enable or disable the service. Handles errors.
    pub(crate) fn on_init(&mut self, world: &mut World) -> Result<(), ServiceErrorKind<E>> {
        debug!("Initializing {}", self.handle);
        if self.initialized {
            let error = ServiceErrorKind::AlreadyInitialized(self.handle.to_string());
            self.on_failure(world, error.clone(), true);
            return Err(error);
        }
        // TODO: on_init should allow asyncronous behavior.
        self.set_state(world, ServiceState::Initializing);

        // initialize dependencies
        let mut deps: Vec<_> = self
            .deps
            .iter_mut()
            .filter(|d| d.is_service() && d.is_initialized())
            .collect();
        for dep in deps.iter_mut() {
            if let Err(e) = dep.initialize(world) {
                let error = ServiceErrorKind::Dependency(
                    self.handle.to_string(),
                    dep.display_name(),
                    e.to_string(),
                );
                self.on_failure(world, error.clone(), false);
                return Err(error);
            }
        }

        // run registered hook
        self.hooks.on_init.initialize(world); // TODO: Does this clear state?
        let res = self.hooks.on_init.run_without_applying_deferred((), world);
        match res {
            Ok(val) => {
                self.initialized = true;
                self.initialized_at = Some(world.change_tick());
                self.last_update = Some(world.change_tick());
                if val {
                    let res = self.on_enable(world);
                    self.hooks.on_init.apply_deferred(world);
                    res
                } else {
                    let res = self.on_disable(world);
                    self.hooks.on_init.apply_deferred(world);
                    res
                }
            }
            Err(error) => {
                let error = ServiceErrorKind::Own(error);
                self.on_failure(world, error.clone(), false);
                self.hooks.on_init.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Enables the service. If it is not already initialized, this will do so.
    pub(crate) fn on_enable(&mut self, world: &mut World) -> Result<(), ServiceErrorKind<E>> {
        debug!("Enabling {}", self.handle);
        if !self.initialized {
            return self.on_init(world);
        }
        self.hooks.on_enable.initialize(world);
        let res = self
            .hooks
            .on_enable
            .run_without_applying_deferred((), world);
        match res {
            Ok(val) => {
                self.set_state(world, ServiceState::Enabled);
                self.hooks.on_enable.apply_deferred(world);
                Ok(val)
            }
            Err(error) => {
                let error = ServiceErrorKind::Own(error);
                self.on_failure(world, error.clone(), false);
                self.hooks.on_enable.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Disables the service if possible.
    pub(crate) fn on_disable(&mut self, world: &mut World) -> Result<(), ServiceErrorKind<E>> {
        debug!("Disabling {}", self.handle);
        if !self.initialized {
            let error = ServiceErrorKind::Uninitialized(self.handle.to_string());
            self.on_failure(world, error.clone(), true);
            return Err(error);
        }
        self.hooks.on_disable.initialize(world);
        let res = self
            .hooks
            .on_disable
            .run_without_applying_deferred((), world);
        match res {
            Ok(val) => {
                self.set_state(world, ServiceState::Disabled);
                self.hooks.on_disable.apply_deferred(world);
                Ok(val)
            }
            Err(error) => {
                let error = ServiceErrorKind::Own(error);
                self.on_failure(world, error.clone(), false);
                self.hooks.on_disable.apply_deferred(world);
                Err(error)
            }
        }
    }

    pub(crate) fn on_update(
        &mut self,
        world: &mut World,
        data: D,
    ) -> Result<(), ServiceErrorKind<E>> {
        self.hooks.on_update.initialize(world);
        let res = self
            .hooks
            .on_update
            .run_without_applying_deferred(data, world);
        match res {
            Ok(data) => {
                self.data = data;
                world.trigger(ServiceUpdated::new(Self::handle()));
                Ok(())
            }
            Err(e) => {
                let error = ServiceErrorKind::Own(e);
                self.on_failure(world, error.clone(), false);
                Err(error)
            }
        }
    }

    pub(crate) fn on_fail_cmd(&mut self, world: &mut World, error: ServiceErrorKind<E>) {
        self.on_failure(world, error, false);
    }

    /// Handles errors. If `is_warning`, the service's state will not change.
    pub(crate) fn on_failure(
        &mut self,
        world: &mut World,
        error: ServiceErrorKind<E>,
        is_warning: bool,
    ) {
        debug!("Failing {}", self.handle);
        self.hooks.on_failure.initialize(world);
        self.hooks
            .on_failure
            .run_without_applying_deferred(error.clone(), world);
        if !is_warning {
            error!("{error}");
            self.set_state(world, ServiceState::Failed(error));
        } else {
            warn!("{error}");
        }
        self.hooks.on_failure.apply_deferred(world);
    }

    pub(crate) fn set_state(&mut self, world: &mut World, state: ServiceState<E>) {
        debug!("Setting {} state: {state:?}", self.handle);
        let old_state = self.state.clone();
        self.state = state.clone();
        world.trigger(ServiceStateChange::<T, D, E>::new((
            old_state.clone(),
            state.clone(),
        )));
        world.trigger(EnterServiceState::<T, D, E>::new(state));
        world.trigger(ExitServiceState::<T, D, E>::new(old_state));
    }
}
