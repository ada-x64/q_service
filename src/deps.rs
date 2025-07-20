use crate::graph::{DagError, DependencyGraph, NodeId};
use crate::prelude::*;
use bevy_asset::{
    Asset, AssetServer, Handle, LoadState, RecursiveDependencyLoadState, UntypedAssetId,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemId;
use tracing::*;

/// This is the underlying data for an [Asset] dependency. Asset dependencies
/// are kept alive by storing a strong handle in an entity,
/// [AssetData::container], which owns a [KeepHandleAlive] component. Note that
/// spinning an asset dep down _does not_ guarantee that the asset is removed
/// from memory, as there may be another active strong handle. Spinning it down
/// only means that _this_ strong handle no longer exists.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[allow(missing_docs, reason = "obvious")]
pub struct AssetData {
    pub id: UntypedAssetId,
    pub name: String,
    pub status: ServiceStatus,
    /// An entity containing a strong handle to the underyling [Asset].
    pub container: Entity,
}

/// This is the underyling data for a [Resource] dependency. Resource deps are
/// literal resources whose lifetimes are equivalent to the service's lifetime.
/// You can define how the resource is initialized and deinitialized using the
/// included init and deinit functions, stored here as entities. These may not
/// be async.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[allow(missing_docs, reason = "obvious")]
pub struct ResourceData {
    pub id: ComponentId,
    pub name: String,
    pub status: ServiceStatus,
    /// The initialisation function, as an Entity.
    pub init: Entity,
    /// The deinitialisation function, as an Entity.
    pub deinit: Entity,
}

/// The main abstraction for service dependencies. This includes the underyling
/// [ServiceData], [ResourceData], and [AssetData].
///
/// All data for services is stored through this abstraction and placed in the
/// [GraphDataCache] resource for global access.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GraphData {
    /// A Service as a service dependency.
    Service(ServiceData),
    /// A Resource as a service dependency.
    Resource(ResourceData),
    /// An Asset as a service dependency.
    Asset(AssetData),
}
impl GraphData {
    /// Create a service dependency.
    pub fn service(data: ServiceData) -> Self {
        Self::Service(data)
    }

    #[allow(missing_docs)]
    pub fn is_service(&self) -> bool {
        matches!(self, Self::Service { .. })
    }

    /// Gets the underlying ServiceData of the node, if the type matches..
    pub fn as_service(&self) -> Option<&ServiceData> {
        if let Self::Service(data) = self {
            Some(data)
        } else {
            None
        }
    }
    /// Gets the underlying ServiceData, mutably.
    pub fn as_service_mut(&mut self) -> Option<&mut ServiceData> {
        if let Self::Service(data) = self {
            Some(data)
        } else {
            None
        }
    }

    /// Create a resource dependency.
    /// Init and deinit systems must impl `IntoSystem<(),(), _>`.
    pub fn resource<R: Resource>(world: &mut World, init: Entity, deinit: Entity) -> Self {
        let id = world.register_resource::<R>();
        Self::Resource(ResourceData {
            id,
            name: name_from_type::<R>(),
            init,
            deinit,
            status: ServiceStatus::uninit(),
        })
    }
    #[allow(missing_docs)]
    pub fn is_resource(&self) -> bool {
        matches!(self, Self::Resource { .. })
    }
    /// Gets the underlying ResourceData of the node, if the type matches..
    pub fn as_resource(&self) -> Option<&ResourceData> {
        if let Self::Resource(data) = self {
            Some(data)
        } else {
            None
        }
    }
    /// Gets the underlying ResourceData, mutably.
    pub fn as_resource_mut(&mut self) -> Option<&mut ResourceData> {
        if let Self::Resource(data) = self {
            Some(data)
        } else {
            None
        }
    }

    /// Create an asset dependency. Will spawn an entity with an AssetContainer
    /// (internal component struct) which will keep the handle alive at least as
    /// long as the service is up.
    pub fn asset<T: Asset, S: Service>(handle: Handle<T>, world: &mut World) -> Self {
        let entity = world.spawn(KeepHandleAlive::<T>(handle.clone())).id();
        Self::Asset(AssetData {
            id: handle.untyped().id(),
            name: name_from_type::<T>(),
            container: entity,
            status: ServiceStatus::uninit(),
        })
    }

    #[allow(missing_docs)]
    pub fn is_asset(&self) -> bool {
        matches!(self, Self::Asset { .. })
    }
    /// Gets the underlying AssetData of the node, if the type matches..
    pub fn as_asset(&self) -> Option<&AssetData> {
        if let Self::Asset(data) = self {
            Some(data)
        } else {
            None
        }
    }
    /// Gets the underlying AssetData, mutably.
    pub fn as_asset_mut(&mut self) -> Option<&mut AssetData> {
        if let Self::Asset(data) = self {
            Some(data)
        } else {
            None
        }
    }
    #[allow(missing_docs)]
    pub fn name(&self) -> &str {
        match self {
            GraphData::Service(ServiceData { name, .. }) => name,
            GraphData::Resource(ResourceData { name, .. }) => name,
            GraphData::Asset(AssetData { name, .. }) => name,
        }
    }
    #[allow(missing_docs)]
    pub fn id(&self) -> NodeId {
        match self {
            GraphData::Service(ServiceData { id, .. }) => *id,
            GraphData::Resource(ResourceData { id, .. }) => NodeId::Resource(*id),
            GraphData::Asset(AssetData { id, .. }) => NodeId::Asset(*id),
        }
    }
    #[allow(missing_docs)]
    pub fn status(&self) -> ServiceStatus {
        match self {
            GraphData::Service(ServiceData { status, .. }) => status.clone(),
            GraphData::Resource(ResourceData { status, .. }) => status.clone(),
            GraphData::Asset(AssetData { status, .. }) => status.clone(),
        }
    }

    /// Initializes or deinitializes the dep.
    /// Called during ServiceData::handle_dep
    pub(crate) fn cycle(
        &mut self,
        world: &mut World,
        down_reason: Option<DownReason>,
    ) -> Result<(), ServiceError> {
        let is_init = down_reason.is_none();
        match self {
            GraphData::Service(service) => cycle_service(world, service, down_reason.clone()),
            GraphData::Resource(ResourceData { init, deinit, .. }) => {
                if is_init {
                    let init: SystemId<(), ()> = SystemId::from_entity(*init);
                    world
                        .run_system(init)
                        .expect("Function signature should match.");
                    Ok(())
                } else {
                    let deinit: SystemId<(), ()> = SystemId::from_entity(*deinit);
                    world
                        .run_system(deinit)
                        .expect("Function signature should match.");
                    Ok(())
                }
            }
            GraphData::Asset(AssetData {
                container, status, ..
            }) => {
                if let Some(reason) = down_reason {
                    // drop container so the strong handle is dropped
                    // NOTE this does not mean the asset is necessarily removed from the world
                    // there might be another strong handle active
                    world.despawn(*container);
                    *status = ServiceStatus::Down(reason);
                }
                Ok(())
            }
        }
    }
}

/// Initialization error for dependencies.
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, Clone)]
pub enum DepInitErr {
    #[error("Service '{0}' failed to initialize with message:\n{1}")]
    Service(String, String),
    #[error("Dependency '{0}' not found.")]
    NotFound(String),
    #[error("Dependency '{0}' depends on itself.")]
    DepLoop(String),
    #[error("Service dependencies contain cycle(s).\n{0}")]
    DepCycle(#[from] DagError),
}

/// Adds a service to the dependency graph. Will fail if cycles are detected.
/// Returns the topsort of the passed in dependencies.
pub(crate) fn register_deps(
    global_graph: &mut DependencyGraph,
    parent: NodeId,
    deps: Vec<NodeId>,
) -> Result<Vec<NodeId>, DepInitErr> {
    // NOTE: We're duplicating the dependency heirarchy here.
    // Could blow up.
    // Ideally the local graphs are just references to the global graph.
    add_and_sort(global_graph, parent, deps)?;
    let topsort = global_graph.subgraph(parent).topsort_graph()?;
    Ok(topsort)
}

fn add_and_sort(
    graph: &mut DependencyGraph,
    parent: NodeId,
    deps: Vec<NodeId>,
) -> Result<(), DepInitErr> {
    graph.add_node(parent);
    for dep in deps {
        graph.add_node(dep);
        graph.add_edge(parent, dep);
    }
    // see if the graph makes sense...
    match graph.topsort_graph() {
        Ok(vec) => {
            graph.topsort = vec;
        }
        Err(e) => {
            let err = if let DagError::DependencyLoop(name) = e {
                DepInitErr::DepLoop(name)
            } else {
                e.into()
            };
            return Err(err);
        }
    }
    Ok(())
}

/// Contains a strong asset handle. Used to keep the asset alive at least as long as the owning service.
#[derive(Component)]
pub struct KeepHandleAlive<T: Asset>(pub Handle<T>);

/// System run every pre-update to check service dependency status. Will update
/// the stored dependency's status.\
/// NOTE: For now, this only updates Asset dependencies, as Service dependencies
/// have their own logic, and Resources are not async.
pub(crate) fn update_dep_status<S: Service>(
    service: ServiceRef<S>,
    asset_server: Res<AssetServer>,
    mut cache: ResMut<GraphDataCache>,
) {
    if service.status.is_down() {
        // don't reawaken the asset dep
        return;
    }
    for dep in service.deps.iter() {
        if let Some(AssetData {
            id, name, status, ..
        }) = cache.get_asset_mut(*dep)
        {
            *status = update_asset_status(&asset_server, *id, name);
        }
    }
}

fn update_asset_status(server: &AssetServer, id: UntypedAssetId, name: &str) -> ServiceStatus {
    let my_load_state = server
        .get_load_state(id)
        .expect("Asset ID should be registered.");
    let dep_load_state = server
        .get_recursive_dependency_load_state(id)
        .expect("Asset ID should be registered.");

    match (my_load_state, dep_load_state) {
        (LoadState::NotLoaded, RecursiveDependencyLoadState::NotLoaded) => {
            ServiceStatus::Down(DownReason::Uninitialized)
        }
        (LoadState::Loaded, RecursiveDependencyLoadState::Loaded) => ServiceStatus::Up,
        (_, RecursiveDependencyLoadState::Failed(asset_load_error)) => {
            ServiceStatus::Down(DownReason::Failed(ServiceError::Dependency(
                name.to_string(),
                asset_load_error.to_string(),
            )))
        }
        (LoadState::Failed(asset_load_error), _) => ServiceStatus::Down(DownReason::Failed(
            ServiceError::Own(asset_load_error.to_string()),
        )),
        _ => ServiceStatus::Init,
    }
}

/// Directly initializes or deinitializes the service dependency.
/// State will be updated on the next update_dep_status call.
fn cycle_service(
    world: &mut World,
    service: &mut ServiceData,
    down_reason: Option<DownReason>,
) -> Result<(), ServiceError> {
    // if the dep is not registered, we can't spin it up
    if !service.registered() {
        return Err(ServiceError::Dependency(
            service.name().to_string(),
            "Service has not been registered.".to_string(),
        ));
    }
    let status = service.status();
    let run = if down_reason.is_none() {
        !status.is_up() && !status.is_initializing()
    } else {
        !status.is_down() && !status.is_deinitializing()
    };
    if run {
        if let Some(reason) = down_reason.clone() {
            match reason {
                DownReason::Failed(error) => service.fail(world, error),
                DownReason::SpunDown => service.spin_down(world),
                _ => {
                    unreachable!()
                }
            }
        } else {
            service.spin_up(world)
        }
    }
    Ok(())
}
