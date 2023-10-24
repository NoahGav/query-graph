use query_graph::{Graph, ResolveQuery};

#[derive(Clone, PartialEq, Eq, Hash)]
enum Query {
    Foo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
}
