use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use enum_as_inner::EnumAsInner;
use query_graph::{Graph, QueryResolver, ResolveQuery};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

#[derive(Clone)]
struct Document {
    path: PathBuf,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SyntaxTree {
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticModel {
    syntax_trees: HashMap<PathBuf, Arc<SyntaxTree>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Query {
    GetAllDocuments,
    GetDocumentContent(PathBuf),
    GetSyntaxTree(PathBuf),
    GetSemanticModel,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
enum QueryResult {
    GetAllDocuments(HashSet<PathBuf>),
    GetDocumentContent(String),
    GetSyntaxTree(Arc<SyntaxTree>),
    GetSemanticModel(Arc<SemanticModel>),
}

struct Compiler {
    snapshot: Arc<Snapshot>,
}

struct Snapshot {
    state: Arc<CompilerState>,
    graph: Arc<Graph<Query, QueryResult>>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            snapshot: Arc::new(Snapshot::new()),
        }
    }

    fn mutate<F: FnOnce(&mut CompilerState)>(&mut self, mutation: F) {
        let mut new_state = self.snapshot.state.as_ref().clone();
        mutation(&mut new_state);
        self.snapshot = Arc::new(self.snapshot.increment(Arc::new(new_state)));
    }

    fn snapshot(&self) -> Arc<Snapshot> {
        self.snapshot.clone()
    }
}

impl Snapshot {
    fn new() -> Self {
        let state = Arc::new(CompilerState::default());

        Self {
            state: state.clone(),
            graph: Graph::new(state),
        }
    }

    fn get_semantic_model(&self) -> Arc<SemanticModel> {
        let result = self.graph.query(Query::GetSemanticModel);
        result.as_get_semantic_model().unwrap().clone()
    }

    fn increment(&self, new_state: Arc<CompilerState>) -> Snapshot {
        Snapshot {
            state: new_state.clone(),
            graph: self.graph.increment(new_state),
        }
    }
}

#[derive(Clone, Default)]
struct CompilerState {
    documents: HashMap<PathBuf, Document>,
}

impl ResolveQuery<Query, QueryResult> for Arc<CompilerState> {
    fn resolve(&self, q: Query, resolver: Arc<QueryResolver<Query, QueryResult>>) -> QueryResult {
        println!("{:?}", q);
        match q {
            Query::GetAllDocuments => QueryResult::GetAllDocuments({
                self.documents.keys().cloned().collect::<HashSet<_>>()
            }),
            Query::GetDocumentContent(path) => QueryResult::GetDocumentContent({
                self.documents.get(&path).unwrap().content.clone()
            }),
            Query::GetSyntaxTree(path) => QueryResult::GetSyntaxTree({
                let content = resolver.query(Query::GetDocumentContent(path));
                let content = content.as_get_document_content().unwrap().clone();

                Arc::new(SyntaxTree { content })
            }),
            Query::GetSemanticModel => QueryResult::GetSemanticModel({
                let documents = resolver.query(Query::GetAllDocuments);
                let documents = documents.as_get_all_documents().unwrap();

                Arc::new(SemanticModel {
                    syntax_trees: documents
                        .par_iter()
                        .map(|path| {
                            let tree = resolver.query(Query::GetSyntaxTree(path.clone()));
                            (path.clone(), tree.as_get_syntax_tree().unwrap().clone())
                        })
                        .collect::<HashMap<_, _>>(),
                })
            }),
        }
    }
}

fn main() {
    let mut compiler = Compiler::new();

    compiler.mutate(|state| {
        state.documents.insert(
            "index.html".into(),
            Document {
                path: "index.html".into(),
                content: "<h1></h1>".into(),
            },
        );
    });

    let snapshot = compiler.snapshot();

    let model = snapshot.get_semantic_model();
    println!("{:#?}", model);

    compiler.mutate(|state| {
        state.documents.insert(
            "index.html".into(),
            Document {
                path: "index.html".into(),
                content: "<h1>Hello, world!</h1>".into(),
            },
        );
    });

    let model = compiler.snapshot().get_semantic_model();
    println!("{:#?}", model);

    // As you can see I can still use the old snapshot and it doesn't
    // resolve any already resolved queries again.
    let model = snapshot.get_semantic_model();
    println!("{:#?}", model);
}
