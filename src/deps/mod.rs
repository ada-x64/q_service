pub(crate) mod graph;

use crate::{
    deps::graph::{DagError, DependencyGraph, NodeId, NodeInfo},
    prelude::*,
};
use bevy_ecs::prelude::*;
use tracing::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ServiceAsDep {
    node_id: NodeId,
    display_name: String,
    is_initialized: bool,
    initialize: fn(&mut World) -> Result<(), DepInitErr>,
}
impl<T, D, E> From<ServiceHandle<T, D, E>> for ServiceAsDep
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn from(handle: ServiceHandle<T, D, E>) -> Self {
        ServiceAsDep {
            node_id: NodeId::service(handle.clone()),
            display_name: handle.to_string(),
            is_initialized: false,
            initialize: Service::<T, D, E>::init_as_dep,
        }
    }
}
impl<T, D, E> From<ServiceHandle<T, D, E>> for ServiceDep
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn from(value: ServiceHandle<T, D, E>) -> Self {
        Self::Service(value.into())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ServiceDep {
    Service(ServiceAsDep),
    // Asset(AssetAsDep),
    // Resource(ResourceAsDep),
}
impl ServiceDep {
    pub fn display_name(&self) -> String {
        match self {
            ServiceDep::Service(service_as_dep) => service_as_dep.display_name.clone(),
        }
    }
    pub fn node_id(&self) -> NodeId {
        match self {
            ServiceDep::Service(service_as_dep) => service_as_dep.node_id.clone(),
        }
    }
    pub fn is_initialized(&self) -> bool {
        match self {
            ServiceDep::Service(service_as_dep) => service_as_dep.is_initialized.clone(),
        }
    }
    pub fn is_service(&self) -> bool {
        matches!(self, ServiceDep::Service(_))
    }
    pub fn initialize(&mut self, world: &mut World) -> Result<(), DepInitErr> {
        match self {
            ServiceDep::Service(service_as_dep) => service_as_dep.initialize.apply(world),
        }
    }
    pub fn node_info(&self) -> NodeInfo {
        NodeInfo {
            display_name: self.display_name(),
        }
    }
}

/// Initialization error for dependencies.
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, Clone)]
pub enum DepInitErr {
    #[error("Service '{0}' failed to initialize with error:\n{1}")]
    Service(String, String),
    #[error("Dependency '{0}' not found.")]
    NotFound(String),
    #[error("Dependency '{0}' depends on itself.")]
    DepLoop(String),
    #[error("Service dependencies contain cycle(s).\n{0}")]
    DepCycle(#[from] DagError),
}

/// Adds a service to the dependency graph. Will fail if cycles are
/// detected.
pub(crate) fn register_service_dep(
    graph: &mut DependencyGraph,
    parent: &ServiceDep,
    deps: Vec<&ServiceDep>,
) -> Result<(), DepInitErr> {
    graph.add_node_from_dep(parent);
    for dep in deps {
        info!(
            "Adding dep: {} ({}) -> {} ({})",
            parent.display_name(),
            parent.node_id(),
            dep.display_name(),
            dep.node_id(),
        );
        graph.add_node_from_dep(dep);
        graph.add_edge(parent.node_id(), dep.node_id());
        // see if the graph makes sense...
        match graph.topsort_graph() {
            Ok(vec) => {
                graph.topsort = vec;
                // would like to iterate here but seems impossible to do
                // register_service_dep(graph, dep.id())?;
            }
            Err(e) => {
                let err = if matches!(e, DagError::DependencyLoop(_)) {
                    DepInitErr::DepLoop(dep.display_name())
                } else {
                    e.into()
                };
                return Err(err);
            }
        }
    }
    Ok(())
}
