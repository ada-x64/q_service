use crate::{
    deps::{GraphData, register_deps},
    graph::DependencyGraph,
    prelude::*,
    spec::ServiceSpec,
};
use bevy_ecs::{component::ComponentId, prelude::*, system::SystemId};
use bevy_platform::prelude::*;
use tracing::{debug, error, warn};

/// The inner Service data structure.
/// Stored globally in the [GraphDataCache].
/// Accessed through the [Service] trait.
#[allow(missing_docs, reason = "obvious")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceData {
    pub name: String,
    pub id: NodeId,
    pub status: ServiceStatus,
    event_queue: Vec<ServiceUpdated>,
    registered: bool,
    /// Service dependencies, stored in topsorted order.
    pub(crate) deps: Vec<NodeId>,
    pub(crate) tasks: Vec<Entity>,
    // SystemIds are Entities + a marker. Can't store the marker so we just have to store the Entity.
    pub(crate) on_init: Option<Entity>,
    pub(crate) on_deinit: Option<Entity>,
    pub(crate) on_up: Option<Entity>,
    pub(crate) on_down: Option<Entity>,
}

impl ServiceData {
    pub(crate) fn new<T: Service>(id: ComponentId) -> Self {
        Self {
            // data,
            status: ServiceStatus::default(),
            on_init: Default::default(),
            on_deinit: Default::default(),
            on_up: Default::default(),
            on_down: Default::default(),
            deps: Vec::new(),
            id: NodeId::Service(id),
            tasks: Vec::new(),
            name: T::name().to_string(),
            registered: false,
            event_queue: Vec::new(),
        }
    }
    /// Inputs: World, ID of the wrapper resource.
    pub(crate) fn register<T: Service>(world: &mut World, spec: ServiceSpec<T>) {
        let on_init = spec
            .on_init
            .map(|hook| world.register_boxed_system(hook.0).entity());
        let on_deinit = spec
            .on_deinit
            .map(|hook| world.register_boxed_system(hook.0).entity());
        let on_up = spec
            .on_up
            .map(|hook| world.register_boxed_system(hook.0).entity());
        let on_down = spec
            .on_down
            .map(|hook| world.register_boxed_system(hook.0).entity());

        let cid = world.resource_id::<T>().unwrap();
        let id = NodeId::Service(cid);
        // insert self into dependency tree.
        let this = Self::new::<T>(cid).clone();
        let mut deps = {
            let mut graph = world.resource_mut::<DependencyGraph>();
            register_deps(&mut graph, this.id, spec.deps).expect("Dependencies are invalid.")
        };
        // remove self from topsort
        assert_eq!(id, deps.remove(0));
        let this = Self {
            on_init,
            on_deinit,
            on_up,
            on_down,
            deps,
            registered: true,
            ..this
        };
        world
            .resource_mut::<GraphDataCache>()
            .insert(id, GraphData::Service(this));
    }

    // Getters, setters ///////////////////////////////////////////////////////

    /// Gets this service's dependencies as [NodeId]s.
    /// Use the [GraphDataCache] to get the particular dependencies.
    pub fn deps(&self) -> &[NodeId] {
        &self.deps
    }

    /// Gets this service's status, owned.
    pub fn status(&self) -> ServiceStatus {
        self.status.clone()
    }

    /// Sets the current status and queues up a broadcast event.
    fn set_status(&mut self, status: ServiceStatus) {
        self.event_queue.push(ServiceUpdated {
            old_status: self.status.clone(),
            new_status: status.clone(),
            id: self.id,
        });
        debug!(
            "({}) NEW STATUS: {:?} -> {status:?}",
            self.name(),
            self.status,
        );
        self.status = status;
    }

    /// Gets this service's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the ID of this ServiceData's [Service] resource.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Returns whether [ServiceAppExt::register_service] was called for this
    /// service's [Service] resource.
    pub fn registered(&self) -> bool {
        self.registered
    }

    // Commands ///////////////////////////////////////////////////////////////

    /// Spins the service up, automatically running its initialization and on_up
    /// hooks. Will do nothing if the service is already up. See
    /// [hooks](crate::lifecycle::hooks) for more details.
    pub fn spin_up(&mut self, world: &mut World) {
        self.initialize(world, false);
    }
    /// Forcibly spins the service up, automatically running its initialization
    /// and on_up hooks. See [hooks](crate::lifecycle::hooks) for more details.
    pub fn restart(&mut self, world: &mut World) {
        self.initialize(world, true);
    }
    /// Spins the service down, automatically running its deinitialization and
    /// on_down hooks. Will do nothing if the service is already down for any
    /// reason. See [hooks](crate::lifecycle::hooks) for more details.
    pub fn spin_down(&mut self, world: &mut World) {
        self.deinit(world, DownReason::SpunDown);
    }
    /// Fails the service with the given error. Will run the deinitialization
    /// and on_down hooks. If the deinit hook fails during this process, the
    /// service will forcibly shut down.
    pub fn fail(&mut self, world: &mut World, error: ServiceError) {
        self.on_failure(world, error, false);
    }

    // Lifecycle ///////////////////////////////////////////////////////////////

    #[tracing::instrument(skip_all, fields(force))]
    fn initialize(&mut self, world: &mut World, force: bool) {
        debug!("({}) Initializing...", self.name());
        if self.status().is_up() && !force {
            warn!(
                "Tried to spin up service {}, but it's already up!",
                self.name,
            );
            return;
        }

        self.set_status(ServiceStatus::Init);

        if let Err(e) = self.cycle_deps(world, None) {
            debug!("({}) deps failed!", self.name());
            return self.on_failure(world, e, false);
        }

        debug!("({}) deps ok", self.name());
        let res: InitResult = self.run_hook(world, self.on_init).unwrap_or(Ok(None));
        match res {
            Ok(Some(task)) => {
                debug!("({}) hook is async", self.name());
                let id = world.spawn(task).id();
                self.tasks.push(id);
            }
            Ok(None) => {
                debug!("({}) hook is sync", self.name());
                match self.deps_ok(ServiceStatus::Up, world.resource::<GraphDataCache>()) {
                    Ok(true) => {
                        debug!("({}) deps all done", self.name());
                        self.on_up(world);
                    }
                    Ok(false) => {}
                    Err(e) => {
                        self.fail(world, e);
                    }
                }
            }
            Err(e) => {
                debug!("({}) hook failed", self.name());
                self.on_failure(world, ServiceError::Own(e.to_string()), false);
            }
        }
        debug!("({}) ... Done Initializing!", self.name());
    }

    /// Should only be run when all deps are finished.
    #[tracing::instrument(skip_all)]
    fn on_up(&mut self, world: &mut World) {
        let res: UpResult = self.run_hook(world, self.on_up).unwrap_or(Ok(()));
        if let Err(error) = res {
            let error = ServiceError::Own(error.to_string());
            self.on_failure(world, error, false);
        } else {
            self.set_status(ServiceStatus::Up);
        }
    }

    #[tracing::instrument(skip_all, fields(reason))]
    fn deinit(&mut self, world: &mut World, reason: DownReason) {
        debug!("({}) Deinitializing... ({reason:?})", self.name());
        let is_failure = matches!(reason, DownReason::Failed(_));
        if !is_failure && self.status().is_down() || is_failure && self.status().is_failed() {
            warn!(
                "Tried to spin down service {}, but it was already down!",
                self.name
            );
            return;
        }

        self.set_status(ServiceStatus::Deinit(reason.clone()));
        if let Err(e) = self.cycle_deps(world, Some(reason.clone())) {
            debug!("({}) cycle_deps failed!", self.name());
            return self.on_failure(world, e, true);
        }

        let res: DeinitResult = self.run_hook(world, self.on_deinit).unwrap_or(Ok(None));
        match res {
            Ok(Some(res)) => {
                debug!("({}) hook is async", self.name());
                let task = world.spawn(res).id();
                self.tasks.push(task);
            }
            Ok(None) => match self.deps_ok(
                ServiceStatus::Down(reason.clone()),
                world.resource::<GraphDataCache>(),
            ) {
                Ok(true) => {
                    debug!("({}) deps all done", self.name());
                    self.on_down(world, reason);
                }
                Ok(false) => {
                    debug!("({}) waiting for deps", self.name());
                }
                Err(e) => {
                    debug!("({}) deps failed", self.name());
                    self.on_failure(world, e, true);
                }
            },
            Err(e) => {
                debug!("({}) hook failed", self.name());
                self.on_failure(world, ServiceError::Own(e.to_string()), true)
            }
        }
        debug!("({}) ... Done Deinitializing!", self.name());
    }

    /// Should only be run when all deps are finished.
    #[tracing::instrument(skip_all, fields(reason))]
    fn on_down(&mut self, world: &mut World, reason: DownReason) {
        self.run_hook_with::<In<DownReason>, ()>(world, self.on_down, reason.clone())
            .unwrap_or_default();
        self.set_status(ServiceStatus::Down(reason));
    }

    /// Handles errors. If `is_warning`, the service's state will not change.
    /// ## Status
    /// if force { * => Down } else { * => Deinit }
    #[tracing::instrument(skip_all, fields(error, force))]
    fn on_failure(&mut self, world: &mut World, error: ServiceError, force: bool) {
        error!("{error}");
        if !force {
            let reason = DownReason::Failed(error);
            self.deinit(world, reason);
        } else {
            self.set_status(ServiceStatus::failed(error));
        }
    }

    // Helpers ////////////////////////////////////////////////////////////////

    fn run_hook<O: 'static>(&mut self, world: &mut World, hook: Option<Entity>) -> Option<O> {
        self.run_hook_with::<(), O>(world, hook, ())
    }

    fn run_hook_with<I: SystemInput + 'static, O: 'static>(
        &mut self,
        world: &mut World,
        hook: Option<Entity>,
        input: I::Inner<'_>,
    ) -> Option<O> {
        hook.map(|hook| {
            let id = SystemId::<I, O>::from_entity(hook);
            world.run_system_with(id, input).expect("Valid system")
        })
    }

    /// Pass without down_reason to spin up.
    fn cycle_deps(
        &mut self,
        world: &mut World,
        down_reason: Option<DownReason>,
    ) -> Result<(), ServiceError> {
        debug!(
            "({}) {} {} dep(s).",
            self.name,
            if down_reason.is_none() {
                "Initializing"
            } else {
                "Deinitializing"
            },
            self.deps.len(),
        );

        for id in self.deps.iter_mut() {
            if let Some(mut dep) = world.resource_mut::<GraphDataCache>().remove(&*id) {
                dep.cycle(world, down_reason.clone())?;
                world.resource_mut::<GraphDataCache>().insert(*id, dep);
            } else {
                return Err(ServiceError::Dependency(
                    format!("{id:?}"),
                    "Dependency not found in cache.".into(),
                ));
            }
        }
        debug!("({}) ...Done!", self.name);
        Ok(())
    }

    fn deps_ok(&self, goal: ServiceStatus, cache: &GraphDataCache) -> Result<bool, ServiceError> {
        let err = self.deps.iter().find_map(|dep| {
            let status = cache.get(dep)?.status();
            let name = cache.get(dep)?.name();
            match status {
                ServiceStatus::Deinit(DownReason::Failed(e))
                | ServiceStatus::Down(DownReason::Failed(e)) => Some((name, e)),
                _ => None,
            }
        });
        if let Some((name, e)) = err {
            return Err(ServiceError::Dependency(name.to_string(), e.to_string()));
        }
        debug!("Checking deps... goal={goal:?}");
        let res = self.deps.iter().all(|dep| {
            let dep = cache.get(dep).unwrap();
            debug!("({:?}) {:?}", dep.name(), dep.status());
            dep.status() == goal
        });
        debug!("... Done! res={res:?}");
        Ok(res)
    }
}

/// Fires when a service is updated. Use this when you only have the service's ID.
#[derive(Event, Clone, PartialEq, Eq, Hash)]
pub struct ServiceUpdated {
    #[allow(missing_docs)]
    pub old_status: ServiceStatus,
    #[allow(missing_docs)]
    pub new_status: ServiceStatus,
    #[allow(missing_docs)]
    pub id: NodeId,
}
impl std::fmt::Debug for ServiceUpdated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "ServiceUpdated ({:?}) {:?} -> {:?}",
            self.id, self.old_status, self.new_status
        ))
    }
}

/// Run every pre-update to check on service dependencies and transition state if needed.
/// SERVICE STATUS SHOULD NOT BE CHANGED FROM OUTSIDE THE SERVICE!
pub(crate) fn update_async_state<S: Service>(world: &mut World) {
    let goal = match world.service_mut::<S>().status() {
        ServiceStatus::Deinit(r) => ServiceStatus::Down(r),
        ServiceStatus::Init => ServiceStatus::Up,
        _ => return,
    };

    world.service_scope::<S, _>(|world, service| {
        match service.deps_ok(goal.clone(), world.resource::<GraphDataCache>()) {
            Ok(true) => {
                if service.tasks.is_empty() {
                    service.set_status(goal.clone());
                }
            }
            Err(e) => service.fail(world, e),
            _ => {}
        }
    })
}

/// Broadcasts events which have been placed in the service's event queue by status updates.
pub(crate) fn broadcast_new_state<S: Service>(mut service: ServiceMut<S>, mut commands: Commands) {
    for event in service.event_queue.drain(..) {
        // broadcast event
        // debug!(
        //     "({}) Broadcasting status update: {:?} -> {:?}",
        //     S::name(),
        //     event.old_status,
        //     event.new_status
        // );
        commands.send_event(event.clone());
        let ServiceUpdated {
            old_status,
            new_status,
            ..
        } = event;
        commands.send_event(ServiceStateChange::<S>::new((
            old_status.clone(),
            new_status.clone(),
        )));
        commands.trigger(ServiceStateChange::<S>::new((
            old_status.clone(),
            new_status.clone(),
        )));
        commands.send_event(EnterServiceState::<S>::new(new_status.clone()));
        commands.trigger(EnterServiceState::<S>::new(new_status.clone()));
        commands.send_event(ExitServiceState::<S>::new(old_status.clone()));
        commands.trigger(ExitServiceState::<S>::new(old_status.clone()));
    }
}
