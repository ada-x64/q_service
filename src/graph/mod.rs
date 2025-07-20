pub(crate) mod tarjan;

use std::fmt::Debug;

use bevy_asset::UntypedAssetId;
use bevy_ecs::{component::ComponentId, resource::Resource};
use bevy_platform::{
    collections::{HashMap, HashSet},
    hash::FixedHasher,
};
use indexmap::IndexMap;
use smallvec::SmallVec;
use thiserror::Error;

/// The ID of a service dependency, as stored in the [DependencyGraph].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeId {
    /// NodeId for a Service. Services are Resources, so this is just a ComponentId.
    Service(ComponentId),
    /// NodeId for a Resource. Resources are stored globally as components, so this is just a ComponentId.
    Resource(ComponentId),
    /// NodeId for an Asset. Just an UntypedAssetId.
    Asset(UntypedAssetId),
}
impl NodeId {
    /// Gets the underlying id for a service.
    pub fn service_id(self) -> Option<ComponentId> {
        if let NodeId::Service(id) = self {
            Some(id)
        } else {
            None
        }
    }
    /// Gets the underlying id for a resource.
    pub fn resource_id(self) -> Option<ComponentId> {
        if let NodeId::Resource(id) = self {
            Some(id)
        } else {
            None
        }
    }
    /// Gets the underlying id for an asset.
    pub fn asset_id(self) -> Option<UntypedAssetId> {
        if let NodeId::Asset(id) = self {
            Some(id)
        } else {
            None
        }
    }
}

/// Compact storage of a [`NodeId`] and a [`Direction`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeIdAndDir(NodeId, Direction);

/// Compact storage of a [`NodeId`] pair.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct NodeIdPair(NodeId, NodeId);

/// Edge direction.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing = 0,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming = 1,
}

impl Direction {
    /// Return the opposite `Direction`.
    #[inline]
    pub fn opposite(self) -> Self {
        match self {
            Self::Outgoing => Self::Incoming,
            Self::Incoming => Self::Outgoing,
        }
    }
}
/// A directed acyclic graph structure used to track service dependencies.
/// Based on [bevy_ecs::schedule::graph]
#[derive(Default, Debug, Resource)]
pub struct DependencyGraph {
    nodes: IndexMap<NodeId, Vec<NodeIdAndDir>, FixedHasher>,
    edges: HashSet<NodeIdPair, FixedHasher>,
    /// A cached topological ordering of the graph.
    pub(crate) topsort: Vec<NodeId>,
}

impl DependencyGraph {
    /// Return the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Add node `n` to the graph if it doesn't already exist.
    pub fn add_node(&mut self, n: NodeId) {
        self.nodes.entry(n).or_default();
    }

    #[inline]
    fn edge_key(key_a: NodeId, key_b: NodeId) -> NodeIdPair {
        NodeIdPair(key_a, key_b)
    }

    /// Remove a node `n` from the graph.
    ///
    /// Computes in **O(N)** time, due to the removal of edges with other nodes.
    pub fn remove_node(&mut self, n: NodeId) {
        let Some(links) = self.nodes.swap_remove(&n) else {
            return;
        };

        let links = links.into_iter();

        for NodeIdAndDir(succ, dir) in links {
            let edge = if dir == Direction::Outgoing {
                NodeIdPair(n, succ)
            } else {
                NodeIdPair(succ, n)
            };
            // remove all successor links
            self.remove_single_edge(succ, n, dir.opposite());
            // Remove all edge values
            self.edges.remove(&edge);
        }
    }

    /// Return `true` if the node is contained in the graph.
    pub fn contains_node(&self, n: NodeId) -> bool {
        self.nodes.contains_key(&n)
    }

    /// Add an edge connecting `a` and `b` to the graph.
    /// For a directed graph, the edge is directed from `a` to `b`.
    ///
    /// Inserts nodes `a` and/or `b` if they aren't already part of the graph.
    pub fn add_edge(&mut self, a: NodeId, b: NodeId) {
        if self.edges.insert(Self::edge_key(a, b)) {
            // insert in the adjacency list if it's a new edge
            self.nodes
                .entry(a)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(NodeIdAndDir(b, Direction::Outgoing));
            if a != b {
                // self loops don't have the Incoming entry
                self.nodes
                    .entry(b)
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(NodeIdAndDir(a, Direction::Incoming));
            }
        }
    }

    /// Remove edge relation from a to b
    ///
    /// Return `true` if it did exist.
    fn remove_single_edge(&mut self, a: NodeId, b: NodeId, dir: Direction) -> bool {
        let Some(sus) = self.nodes.get_mut(&a) else {
            return false;
        };

        let Some(index) = sus
            .iter()
            .copied()
            .position(|elt| (elt == NodeIdAndDir(b, dir)))
        else {
            return false;
        };

        sus.swap_remove(index);
        true
    }

    /// Remove edge from `a` to `b` from the graph.
    ///
    /// Return `false` if the edge didn't exist.
    pub fn _remove_edge(&mut self, a: NodeId, b: NodeId) -> bool {
        let exist1 = self.remove_single_edge(a, b, Direction::Outgoing);
        let exist2 = if a != b {
            self.remove_single_edge(b, a, Direction::Incoming)
        } else {
            exist1
        };
        let weight = self.edges.remove(&Self::edge_key(a, b));
        debug_assert!(exist1 == exist2 && exist1 == weight);
        weight
    }

    /// Return `true` if the edge connecting `a` with `b` is contained in the
    /// graph.
    pub fn _contains_edge(&self, a: NodeId, b: NodeId) -> bool {
        self.edges.contains(&Self::edge_key(a, b))
    }

    /// Return an iterator over the nodes of the graph.
    pub fn nodes(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId> + ExactSizeIterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    /// Return an iterator of all nodes with an edge starting from `a`.
    pub fn neighbors(&self, a: NodeId) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .filter_map(|NodeIdAndDir(n, dir)| (dir == Direction::Outgoing).then_some(n))
    }

    /// Return an iterator of all neighbors that have an edge between them and
    /// `a`, in the specified direction.
    /// If the graph's edges are undirected, this is equivalent to
    /// *.neighbors(a)*.
    pub fn _neighbors_directed(
        &self,
        a: NodeId,
        dir: Direction,
    ) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .filter_map(move |NodeIdAndDir(n, d)| (d == dir || n == a).then_some(n))
    }

    /// Return an iterator of target nodes with an edge starting from `a`,
    /// paired with their respective edge weights.
    pub fn _edges(&self, a: NodeId) -> impl DoubleEndedIterator<Item = (NodeId, NodeId)> + '_ {
        self.neighbors(a)
            .map(move |b| match self.edges.get(&Self::edge_key(a, b)) {
                None => unreachable!(),
                Some(_) => (a, b),
            })
    }

    /// Return an iterator of target nodes with an edge starting from `a`,
    /// paired with their respective edge weights.
    pub fn _edges_directed(
        &self,
        a: NodeId,
        dir: Direction,
    ) -> impl DoubleEndedIterator<Item = (NodeId, NodeId)> + '_ {
        self._neighbors_directed(a, dir).map(move |b| {
            let (a, b) = if dir == Direction::Incoming {
                (b, a)
            } else {
                (a, b)
            };

            match self.edges.get(&Self::edge_key(a, b)) {
                None => unreachable!(),
                Some(_) => (a, b),
            }
        })
    }

    /// Return an iterator over all edges of the graph with their weight in
    /// arbitrary order.
    pub fn all_edges(&self) -> impl ExactSizeIterator<Item = NodeIdPair> + '_ {
        self.edges.iter().copied()
    }

    pub(crate) fn to_index(&self, ix: NodeId) -> usize {
        self.nodes.get_index_of(&ix).unwrap()
    }

    /// Iterate over all *Strongly Connected Components* in this graph.
    pub(crate) fn iter_sccs(&self) -> impl Iterator<Item = SmallVec<[NodeId; 4]>> + '_ {
        tarjan::new_tarjan_scc(self)
    }

    /// Tries to topologically sort `graph`.
    ///
    /// If the graph is acyclic, returns [`Ok`] with the list of [`NodeId`] in a
    /// valid topological order. If the graph contains cycles, returns
    /// [`Err`] with the list of strongly-connected components that contain
    /// cycles (also in a valid topological order).
    ///
    /// # Errors
    ///
    /// If the graph contain cycles, then an error is returned.
    pub fn topsort_graph(&self) -> Result<Vec<NodeId>, DagError> {
        // Check explicitly for self-edges.
        // `iter_sccs` won't report them as cycles because they still form
        // components of one node.
        if self
            .all_edges()
            .any(|NodeIdPair(left, right)| left == right)
        {
            let error = DagError::DependencyLoop(String::new());
            return Err(error);
        }

        // Tarjan's SCC algorithm returns elements in *reverse* topological
        // order.
        let mut top_sorted_nodes = Vec::with_capacity(self.node_count());
        let mut sccs_with_cycles = Vec::new();

        for scc in self.iter_sccs() {
            // A strongly-connected component is a group of nodes who can all
            // reach each other through one or more paths. If an SCC
            // contains more than one node, there must be
            // at least one cycle within them.
            top_sorted_nodes.extend_from_slice(&scc);
            if scc.len() > 1 {
                sccs_with_cycles.push(scc);
            }
        }

        if sccs_with_cycles.is_empty() {
            // reverse to get topological order
            top_sorted_nodes.reverse();
            Ok(top_sorted_nodes)
        } else {
            let mut cycles = Vec::new();
            for scc in &sccs_with_cycles {
                cycles.append(&mut simple_cycles_in_component(self, scc));
            }
            let error =
                DagError::DependencyCycle(self.get_dependency_cycles_error_message(&cycles));

            Err(error)
        }
    }

    fn get_dependency_cycles_error_message(&self, cycles: &[Vec<NodeId>]) -> String {
        use std::fmt::Write;
        let mut message = format!("Service has {} before/after cycle(s):\n", cycles.len());
        for (i, cycle) in cycles.iter().enumerate() {
            let mut names = cycle.iter();
            let first_name = names.next().unwrap();
            writeln!(
                message,
                "cycle {}: `{first_name:?}` must run before itself",
                i + 1,
            )
            .unwrap();
            writeln!(message, "`{first_name:?}`").unwrap();
            for name in names.chain(core::iter::once(first_name)) {
                writeln!(message, " ... which must run before `{name:?}`").unwrap();
            }
            writeln!(message).unwrap();
        }

        message
    }

    fn color(&self, subgraph: &mut DependencyGraph, parent: NodeId) {
        self.neighbors(parent).for_each(|neighbor| {
            if subgraph.contains_node(neighbor) {
                return;
            }
            subgraph.add_node(neighbor);
            subgraph.add_edge(parent, neighbor);
            self.color(subgraph, neighbor);
        })
    }

    pub(crate) fn subgraph(&self, node: NodeId) -> DependencyGraph {
        let mut subgraph = DependencyGraph {
            nodes: IndexMap::default(),
            edges: HashSet::default(),
            topsort: Vec::default(),
        };
        subgraph.add_node(node);
        self.color(&mut subgraph, node);
        subgraph
    }
}

/// Returns the simple cycles in a strongly-connected component of a directed
/// graph.
///
/// The algorithm implemented comes from
/// ["Finding all the elementary circuits of a directed graph"][1] by D. B.
/// Johnson.
///
/// [1]: https://doi.org/10.1137/0204007
pub fn simple_cycles_in_component(graph: &DependencyGraph, scc: &[NodeId]) -> Vec<Vec<NodeId>> {
    let mut cycles = vec![];
    let mut sccs = vec![SmallVec::from_slice(scc)];

    while let Some(mut scc) = sccs.pop() {
        // only look at nodes and edges in this strongly-connected component
        let mut subgraph = DependencyGraph::default();
        for &node in &scc {
            subgraph.add_node(node);
        }

        for &node in &scc {
            for successor in graph.neighbors(node) {
                if subgraph.contains_node(successor) {
                    subgraph.add_edge(node, successor);
                }
            }
        }

        // path of nodes that may form a cycle
        let mut path = Vec::with_capacity(subgraph.node_count());
        // we mark nodes as "blocked" to avoid finding permutations of the same
        // cycles
        let mut blocked: HashSet<_> =
            HashSet::with_capacity_and_hasher(subgraph.node_count(), Default::default());
        // connects nodes along path segments that can't be part of a cycle
        // (given current root) those nodes can be unblocked at the same
        // time
        let mut unblock_together: HashMap<NodeId, HashSet<NodeId>> =
            HashMap::with_capacity_and_hasher(subgraph.node_count(), Default::default());
        // stack for unblocking nodes
        let mut unblock_stack = Vec::with_capacity(subgraph.node_count());
        // nodes can be involved in multiple cycles
        let mut maybe_in_more_cycles: HashSet<NodeId> =
            HashSet::with_capacity_and_hasher(subgraph.node_count(), Default::default());
        // stack for DFS
        let mut stack = Vec::with_capacity(subgraph.node_count());

        // we're going to look for all cycles that begin and end at this node
        let root = scc.pop().unwrap();
        // start a path at the root
        path.clear();
        path.push(root);
        // mark this node as blocked
        blocked.insert(root);

        // DFS
        stack.clear();
        stack.push((root, subgraph.neighbors(root)));
        while !stack.is_empty() {
            let &mut (ref node, ref mut successors) = stack.last_mut().unwrap();
            if let Some(next) = successors.next() {
                if next == root {
                    // found a cycle
                    maybe_in_more_cycles.extend(path.iter());
                    cycles.push(path.clone());
                } else if !blocked.contains(&next) {
                    // first time seeing `next` on this path
                    maybe_in_more_cycles.remove(&next);
                    path.push(next);
                    blocked.insert(next);
                    stack.push((next, subgraph.neighbors(next)));
                    continue;
                } else {
                    // not first time seeing `next` on this path
                }
            }

            if successors.peekable().peek().is_none() {
                if maybe_in_more_cycles.contains(node) {
                    unblock_stack.push(*node);
                    // unblock this node's ancestors
                    while let Some(n) = unblock_stack.pop() {
                        if blocked.remove(&n) {
                            let unblock_predecessors = unblock_together.entry(n).or_default();
                            unblock_stack.extend(unblock_predecessors.iter());
                            unblock_predecessors.clear();
                        }
                    }
                } else {
                    // if its descendants can be unblocked later, this node will
                    // be too
                    for successor in subgraph.neighbors(*node) {
                        unblock_together.entry(successor).or_default().insert(*node);
                    }
                }

                // remove node from path and DFS stack
                path.pop();
                stack.pop();
            }
        }

        drop(stack);

        // remove node from subgraph
        subgraph.remove_node(root);

        // divide remainder into smaller SCCs
        sccs.extend(subgraph.iter_sccs().filter(|scc| scc.len() > 1));
    }

    cycles
}

/// Category of errors encountered during schedule construction.
#[derive(Error, Debug, Clone)]
pub enum DagError {
    /// A dependency has been told to run before itself.
    #[error("Service `{0}` depends on itself.")]
    DependencyLoop(String),
    /// The dependency graph contains a cycle.
    #[error("Service dependencies contain cycle(s).\n{0}")]
    DependencyCycle(String),
}
