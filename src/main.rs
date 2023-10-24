use std::sync::Arc;

use query_graph::{Graph, QueryResolver, ResolveQuery};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Query {
    Foo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum QueryResult {
    Foo(String),
}

struct State;

impl ResolveQuery<Query, QueryResult> for State {
    fn resolve(&self, q: Query, _resolver: Arc<QueryResolver<Query, QueryResult>>) -> QueryResult {
        println!("Resolving.");
        std::thread::sleep(std::time::Duration::from_secs(3));

        match q {
            Query::Foo => QueryResult::Foo("Foo".into()),
        }
    }
}

fn main() {
    let graph = Graph::new(State);

    let graph_clone = graph.clone();
    let handle = std::thread::spawn(move || graph_clone.query(Query::Foo));

    std::thread::sleep(std::time::Duration::from_secs(1));

    let new_graph = graph.increment(State);
    println!("{:#?}", new_graph);

    new_graph.query(Query::Foo);

    handle.join().unwrap();
}
