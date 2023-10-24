use query_graph::{Graph, ResolveQuery};

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
