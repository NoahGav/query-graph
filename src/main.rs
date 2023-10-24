use std::sync::Arc;

use query_graph::{Graph, QueryResolver, ResolveQuery};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

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
        match q {
            Query::Foo => QueryResult::Foo({
                let bar = resolver.query(Query::Bar);
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

    let start = std::time::Instant::now();
    let count = 100000;

    graph.query(Query::Foo);

    (0..count).into_par_iter().for_each(|_| {
        let new_graph = graph.increment(State);
        // println!("{:#?}", new_graph);
        new_graph.query(Query::Foo);
    });

    println!("{:?}", (std::time::Instant::now() - start) / count);

    // let mut threads = vec![];

    // for _ in 0..100 {
    //     let graph = graph.clone();

    //     threads.push(std::thread::spawn(move || {
    //         let new_graph = graph.increment(State);
    //         new_graph.query(Query::Foo)
    //     }));
    // }

    // for thread in threads {
    //     thread.join().unwrap();
    // }

    // let graph_clone = graph.clone();
    // let handle = std::thread::spawn(move || graph_clone.query(Query::Foo));

    // std::thread::sleep(std::time::Duration::from_secs(1));

    // let new_graph = graph.increment(State);
    // println!("{:#?}", new_graph);

    // new_graph.query(Query::Foo);
    // println!("{:#?}", new_graph);

    // handle.join().unwrap();
}
