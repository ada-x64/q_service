//! The main service module. Defines the Service resource.

use crate::{deps::IsServiceDep, prelude::*};
use bevy_ecs::prelude::*;
use bevy_platform::prelude::*;
use tracing::*;

#[derive(Debug, Resource)]
/// Resource which represents a service.
pub struct Service<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    /// Arbitrary data store.
    pub data: D,
    /// Lifecycle hooks.
    pub hooks: ServiceHooks<E>,
    /// The current state of the service.
    pub state: ServiceState<E>,
    /// Has this service been initialized?
    pub initialized: bool,
    pub(crate) deps: Vec<Box<dyn IsServiceDep>>,
    handle: ServiceHandle<T, D, E>,
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

    pub(crate) fn from_spec(spec: ServiceSpec<T, D, E>) -> Self {
        Self {
            data: spec.initial_data.unwrap_or_default(),
            state: ServiceState::default(),
            hooks: ServiceHooks {
                on_init: spec.on_init.unwrap_or_default(),
                on_enable: spec.on_enable.unwrap_or_default(),
                on_disable: spec.on_disable.unwrap_or_default(),
                on_failure: spec.on_failure.unwrap_or_default(),
            },
            handle: ServiceHandle::const_default(),
            deps: spec.deps,
            initialized: false,
        }
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
        for dep in self.deps.iter_mut() {
            let info = dep.info(world);
            if !info.is_service || info.is_initialized {
                continue;
            }
            if let Err(e) = dep.initialize(world) {
                let error = ServiceErrorKind::Dependency(
                    ServiceHandle::from_service(self).to_string(),
                    info.display_name,
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
