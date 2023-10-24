use std::{
    hash::Hash,
    sync::{Arc, OnceLock},
};

use map::ConcurrentMap;

pub mod map;

#[derive(Debug)]
struct Node<R> {
    result: R,
    changed: bool,
    // TODO: edges.
}

pub struct Graph<Q, R> {
    /// The new map is used for all the queries in this iteration.
    /// This map always starts empty.
    new: ConcurrentMap<Q, Arc<OnceLock<Node<R>>>>,
    /// The old map is used for the queries from the previous iteration.
    /// This maps is a clone of the new map from the previous iteration.
    old: ConcurrentMap<Q, Arc<OnceLock<Node<R>>>>,
    /// The resolver used to resolve queries. The resolver can have it's
    /// own state as long as it's Sync + Send.
    resolver: Box<dyn ResolveQuery<Q, R>>,
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

    fn resolve(self: &Arc<Self>, q: Q) -> Node<R> {
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
                let result = self.resolver.resolve(q);

                Node {
                    // We need to check again if the node is still unresolved. Because
                    // if it isn't we can set changed: old_result != result. Otherwise,
                    // we always set changed to true.
                    changed: match old.get() {
                        Some(old_node) => result != old_node.result,
                        None => true,
                    },
                    result,
                }
            }
        } else {
            // If the node isn't in the old map then the query is new and resolved
            // from scratch.
            let result = self.resolver.resolve(q);

            Node {
                result,
                // Since this is a new node, changed is set to false.
                changed: false,
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

pub trait ResolveQuery<Q, R>: Send + Sync {
    fn resolve(&self, q: Q) -> R;
}
