use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use enum_as_inner::EnumAsInner;
use query_graph::{Graph, QueryResolver, ResolveQuery};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

struct Document {
    path: PathBuf,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntaxTree {
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Query {
    GetAllDocuments,
    GetSyntaxTree(PathBuf),
    GetSemanticModel,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
enum QueryResult {
    GetAllDocuments(HashSet<PathBuf>),
    GetSyntaxTree(SyntaxTree),
    GetSemanticModel(Vec<SyntaxTree>),
}

#[derive(Default)]
struct CompilerState {
    documents: HashMap<PathBuf, Document>,
}

impl ResolveQuery<Query, QueryResult> for CompilerState {
    fn resolve(&self, q: Query, resolver: Arc<QueryResolver<Query, QueryResult>>) -> QueryResult {
        match q {
            Query::GetAllDocuments => QueryResult::GetAllDocuments({
                self.documents.keys().cloned().collect::<HashSet<_>>()
            }),
            Query::GetSyntaxTree(path) => QueryResult::GetSyntaxTree(SyntaxTree {
                content: self.documents.get(&path).unwrap().content.clone(),
            }),
            Query::GetSemanticModel => QueryResult::GetSemanticModel({
                let documents = resolver.query(Query::GetAllDocuments);
                let documents = documents.as_get_all_documents().unwrap();

                documents
                    .par_iter()
                    .map(|path| {
                        let tree = resolver.query(Query::GetSyntaxTree(path.clone()));
                        tree.as_get_syntax_tree().unwrap().clone()
                    })
                    .collect::<Vec<_>>()
            }),
        }
    }
}

fn main() {
    let mut state = CompilerState::default();

    state.documents.insert(
        "index.html".into(),
        Document {
            path: "index.html".into(),
            content: "<h1></h1>".into(),
        },
    );

    let graph = Graph::new(state);

    let model = graph.query(Query::GetSemanticModel);
    let model = model.as_get_semantic_model().unwrap();
    println!("{:#?}", model);

    let mut state = CompilerState::default();

    state.documents.insert(
        "index.html".into(),
        Document {
            path: "index.html".into(),
            content: "<h1>Hello, world!</h1>".into(),
        },
    );

    let graph = graph.increment(state);

    let model = graph.query(Query::GetSemanticModel);
    let model = model.as_get_semantic_model().unwrap();
    println!("{:#?}", model);
}
