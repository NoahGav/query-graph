use std::{
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, OnceLock},
};

use hashbrown::HashSet;
use map::ConcurrentMap;

pub mod map;

#[derive(Debug)]
struct Node<Q, R> {
    result: R,
    changed: bool,
    edges_from: HashSet<Q>,
}

pub struct Graph<Q, R> {
    /// The new map is used for all the queries in this iteration.
    /// This map always starts empty.
    new: ConcurrentMap<Q, Arc<OnceLock<Node<Q, R>>>>,
    /// The old map is used for the queries from the previous iteration.
    /// This maps is a clone of the new map from the previous iteration.
    old: ConcurrentMap<Q, Arc<OnceLock<Node<Q, R>>>>,
    /// The resolver used to resolve queries. The resolver can have it's
    /// own state as long as it's Sync + Send.
    resolver: Box<dyn ResolveQuery<Q, R>>,
}

impl<Q: Debug + Clone + Eq + Hash, R: Debug + Clone> Debug for Graph<Q, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("new", &self.new)
            .field("old", &self.old)
            .finish()
    }
}

impl<Q: Clone + Eq + Hash, R: Clone + Eq> Graph<Q, R> {
    pub fn new(resolver: impl ResolveQuery<Q, R> + 'static) -> Arc<Self> {
        Arc::new(Self {
            new: ConcurrentMap::new(),
            old: ConcurrentMap::new(),
            resolver: Box::new(resolver),
        })
    }

    pub fn query(self: &Arc<Self>, q: Q) -> R {
        let node = self
            .new
            .get_or_insert(q.clone(), || Arc::new(OnceLock::default()));

        let node = node.get_or_init(|| self.resolve(q));
        node.result.clone()
    }

    fn resolve(self: &Arc<Self>, q: Q) -> Node<Q, R> {
        let old = self.old.get(&q);

        if let Some(old) = old {
            // Since there was an old node we have to validate it.
            let old_node = old.get();

            if let Some(_old_node) = old_node {
                // TODO: Since the old node is already resolved we actually validate
                // TODO: the node. To do this, we traverse the graph upwards from this
                // TODO: node to it's dependencies. We check if any of it's dependencies
                // TODO: are changed. If none are we simply reuse the result from the
                // TODO: old_node and set changed to false. Otherwise, we have to resolve
                // TODO: the node again and then compare the new result with the old result.
                todo!()
            } else {
                // Since the old node is not resolved yet we will just resolve
                // it from scratch.
                let resolver = Arc::new(QueryResolver::new(self.clone()));
                let result = self.resolver.resolve(q, resolver.clone());

                Node {
                    // We need to check again if the old node is still unresolved. Because
                    // if it isn't we can set changed to old_result != result. Otherwise,
                    // we always set changed to true.
                    changed: match old.get() {
                        Some(old_node) => result != old_node.result,
                        None => true,
                    },
                    result,
                    edges_from: resolver.edges_from.take(),
                }
            }
        } else {
            // Since the node isn't in the old map then the query is new and resolved
            // from scratch.
            let resolver = Arc::new(QueryResolver::new(self.clone()));
            let result = self.resolver.resolve(q, resolver.clone());

            Node {
                result,
                // Since this is a new node, changed is always false.
                changed: false,
                edges_from: resolver.edges_from.take(),
            }
        }
    }

    pub fn increment(self: &Arc<Self>, resolver: impl ResolveQuery<Q, R> + 'static) -> Arc<Self> {
        let old = self.new.clone();

        Arc::new(Self {
            new: ConcurrentMap::new(),
            old,
            resolver: Box::new(resolver),
        })
    }
}

pub struct QueryResolver<Q, R> {
    graph: Arc<Graph<Q, R>>,
    edges_from: RefCell<HashSet<Q>>,
}

impl<Q: Clone + Eq + Hash, R: Clone + Eq> QueryResolver<Q, R> {
    fn new(graph: Arc<Graph<Q, R>>) -> Self {
        Self {
            graph,
            edges_from: RefCell::new(HashSet::new()),
        }
    }

    pub fn query(&self, q: Q) -> R {
        let result = self.graph.query(q.clone());
        self.edges_from.borrow_mut().insert(q);
        // TODO: edges_to (maybe?).
        result
    }
}

pub trait ResolveQuery<Q, R>: Send + Sync {
    fn resolve(&self, q: Q, resolve: Arc<QueryResolver<Q, R>>) -> R;
}
