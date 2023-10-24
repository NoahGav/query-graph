use std::{
    hash::Hash,
    sync::{Arc, OnceLock},
};

use map::ConcurrentMap;

pub mod map;

#[derive(Debug)]
struct Node<R> {
    result: R,
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

impl<Q: Clone + Eq + Hash, R: Clone> Graph<Q, R> {
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
        let result = self.resolver.resolve(q);

        Node { result }
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
