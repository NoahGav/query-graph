use query_graph::map::ConcurrentMap;

fn main() {
    let map = ConcurrentMap::<i32, i32>::new();

    // map.insert(112378, 1);

    let value = map.get_or_insert(112378, || 2);
    println!("{}", value);
}
