use std::sync::Arc;

use query_graph::{Graph, QueryResolver, ResolveQuery};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Query {
    Foo,
    Bar,
    FooBar(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum QueryResult {
    Foo(String),
    Bar,
    FooBar,
}

struct State;

impl ResolveQuery<Query, QueryResult> for State {
    fn resolve(&self, q: Query, resolver: Arc<QueryResolver<Query, QueryResult>>) -> QueryResult {
        println!("Resolving.");

        match q {
            Query::Foo => QueryResult::Foo({
                let bar = resolver.query(Query::Bar);
                std::thread::sleep(std::time::Duration::from_secs(3));
                format!("Foo{:?}", bar)
            }),
            Query::Bar => {
                resolver.query(Query::FooBar(0));
                resolver.query(Query::FooBar(1));
                resolver.query(Query::FooBar(2));

                QueryResult::Bar
            }
            Query::FooBar(_) => QueryResult::FooBar,
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
    println!("{:#?}", new_graph);

    handle.join().unwrap();
}
