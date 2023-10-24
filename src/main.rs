use std::{
    hash::Hash,
    sync::{Arc, OnceLock},
};

use query_graph::map::ConcurrentMap;

#[derive(Debug)]
struct Node<R> {
    result: R,
}

struct Graph<Q, R> {
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
    fn new(resolver: impl ResolveQuery<Q, R> + 'static) -> Arc<Self> {
        Arc::new(Self {
            new: ConcurrentMap::new(),
            old: ConcurrentMap::new(),
            resolver: Box::new(resolver),
        })
    }

    fn query(self: &Arc<Self>, q: Q) -> R {
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

    fn increment(self: &Arc<Self>, resolver: impl ResolveQuery<Q, R> + 'static) -> Arc<Self> {
        let old = self.new.clone();

        Arc::new(Self {
            new: ConcurrentMap::new(),
            old,
            resolver: Box::new(resolver),
        })
    }
}

trait ResolveQuery<Q, R>: Send + Sync {
    fn resolve(&self, q: Q) -> R;
}

// ==================================================================== \\

#[derive(Clone, PartialEq, Eq, Hash)]
enum Query {
    Foo,
}

#[derive(Debug, Clone)]
enum QueryResult {
    Foo(String),
}

struct State;

impl ResolveQuery<Query, QueryResult> for State {
    fn resolve(&self, q: Query) -> QueryResult {
        println!("Resolving.");
        match q {
            Query::Foo => QueryResult::Foo("Foo".into()),
        }
    }
}

fn main() {
    let state = State;
    let graph = Graph::new(state);

    let mut threads = vec![];

    for _ in 0..100 {
        let graph = graph.clone();
        threads.push(std::thread::spawn(move || graph.query(Query::Foo)));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    // Ok, so the nodes will be stored as a once-cell. The reason for this
    // is so that we do not hold a write lock into the map when resolving
    // the node inside the get_or_insert value method.

    // So, the way the concurrent dependency graph works is fairly simple.
    // It has a valid map of nodes, and an invalid map of nodes. When querying
    // the graph it does get_or_insert on the valid map and then get_or_init
    // on the cell returned. The value in the cell function checks the value
    // of the old map to see if it exists (using get so it doesn't block).
    // If the value isn't ready we simply resolve the query from scratch.
    // If the value is ready then we validate it recursively upwards (using
    // it's edges). If the query was resolved from scratch we then attempt
    // to get the old nodes value again, if it has something now we compare
    // to the new value to see if the value actually changed. If it's still
    // not there we set changed to true (this is because we cannot know
    // whether or not the value is the same or not). If the node wasn't in
    // the old map at all then we just resolve it like normal.
}
