use barq_core::{Document, DocumentId};
use clap::{Parser, Subcommand};
use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "barq-cli")]
#[command(about = "BARQ-NOSQL CLI Client", long_about = None)]
struct Cli {
    #[arg(long, default_value = "http://localhost:7070")]
    host: String,

    #[arg(long, default_value_t = 7070)]
    port: u16,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Connect {
        #[arg(long)]
        host: Option<String>,
        
        #[arg(long)]
        port: Option<u16>,
    },
    Insert {
        #[arg(long)]
        collection: String,
        
        #[arg(long)]
        doc: String,
    },
    Get {
        #[arg(long)]
        collection: String,
        
        #[arg(long)]
        id: String,
    },
    Query {
        #[arg(long)]
        collection: String,
        
        #[arg(long)]
        filter: Option<String>,
    },
    Vector {
        #[arg(long)]
        collection: String,
        
        #[arg(long)]
        field: String,
        
        #[arg(long)]
        query: String,
        
        #[arg(long, default_value_t = 10)]
        k: usize,
    },
    Graph {
        #[arg(long)]
        from: String,
        
        #[arg(long, default_value_t = 1)]
        hops: usize,
        
        #[arg(long)]
        label: Option<String>,
    },
    Bench {
        #[arg(long, default_value_t = 1000)]
        ops: usize,
        
        #[arg(long, default_value_t = 1)]
        threads: usize,
    },
    CreateCollection {
        #[arg(long)]
        name: String,
    },
    Health,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    #[serde(flatten)]
    data: T,
    #[serde(default)]
    error: Option<String>,
}

struct AppState {
    host: String,
    port: u16,
    client: Client,
}

impl AppState {
    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

async fn connect(state: &Mutex<AppState>, host: Option<String>, port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let mut s = state.lock().unwrap();
    if let Some(h) = host {
        s.host = h;
    }
    if let Some(p) = port {
        s.port = p;
    }
    println!("Connected to {}", s.base_url());
    Ok(())
}

async fn insert(state: &Mutex<AppState>, collection: String, doc_str: String) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    let doc: Document = serde_json::from_str(&doc_str)?;
    
    let url = format!("{}/collections/{}/documents", s.base_url(), collection);
    let response = s.client.post(&url).json(&doc).send().await?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("{}", format!("Inserted: {:?}", result).green());
    } else {
        let error: serde_json::Value = response.json().await?;
        println!("{}", format!("Error: {:?}", error).red());
    }
    
    Ok(())
}

async fn get_doc(state: &Mutex<AppState>, collection: String, id: String) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    
    let url = format!("{}/collections/{}/documents/{}", s.base_url(), collection, id);
    let response = s.client.get(&url).send().await?;
    
    if response.status().is_success() {
        let doc: Document = response.json().await?;
        println!("{}", serde_json::to_string_pretty(&doc)?.green());
    } else {
        println!("{}", "Document not found".red());
    }
    
    Ok(())
}

async fn query(state: &Mutex<AppState>, collection: String, filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    
    let query_obj: HashMap<String, serde_json::Value> = if let Some(f) = filter {
        serde_json::from_str(&f)?
    } else {
        HashMap::new()
    };
    
    let query = serde_json::json!({
        "collection": collection,
        "filter": query_obj,
        "limit": 100
    });
    
    let url = format!("{}/collections/{}/query", s.base_url(), collection);
    let response = s.client.post(&url).json(&query).send().await?;
    
    if response.status().is_success() {
        let docs: Vec<Document> = response.json().await?;
        println!("{}", format!("Found {} documents", docs.len()).green());
        for doc in docs {
            println!("{}", serde_json::to_string_pretty(&doc)?);
        }
    } else {
        let error: serde_json::Value = response.json().await?;
        println!("{}", format!("Error: {:?}", error).red());
    }
    
    Ok(())
}

async fn vector_search(
    state: &Mutex<AppState>,
    collection: String,
    field: String,
    query: String,
    k: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    
    let query_vec: Vec<f32> = serde_json::from_str(&query)?;
    
    let request = serde_json::json!({
        "field": field,
        "query": query_vec,
        "k": k
    });
    
    let url = format!("{}/collections/{}/vector", s.base_url(), collection);
    let response = s.client.post(&url).json(&request).send().await?;
    
    if response.status().is_success() {
        let results: Vec<serde_json::Value> = response.json().await?;
        println!("{}", format!("Found {} results", results.len()).green());
        for result in results {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    } else {
        let error: serde_json::Value = response.json().await?;
        println!("{}", format!("Error: {:?}", error).red());
    }
    
    Ok(())
}

async fn graph_traverse(
    state: &Mutex<AppState>,
    from: String,
    hops: usize,
    label: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    
    let request = serde_json::json!({
        "from": from,
        "hops": hops,
        "label": label
    });
    
    let url = format!("{}/collections/graph/traverse", s.base_url());
    let response = s.client.post(&url).json(&request).send().await?;
    
    if response.status().is_success() {
        let results: Vec<serde_json::Value> = response.json().await?;
        println!("{}", format!("Found {} neighbors", results.len()).green());
        for result in results {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    } else {
        let error: serde_json::Value = response.json().await?;
        println!("{}", format!("Error: {:?}", error).red());
    }
    
    Ok(())
}

async fn bench(
    state: &Mutex<AppState>,
    ops: usize,
    threads: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    
    println!("{}", format!("Running benchmark: {} ops with {} threads", ops, threads).yellow());
    
    let mut handles = vec![];
    
    for _ in 0..threads {
        let client = s.client.clone();
        let base_url = s.base_url();
        let ops_per_thread = ops / threads;
        
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            
            for i in 0..ops_per_thread {
                let doc = serde_json::json!({
                    "id": uuid::Uuid::new_v4(),
                    "data": {
                        "index": i,
                        "value": format!("bench_{}", i)
                    }
                });
                
                let url = format!("{}/collections/bench/documents", base_url);
                let _ = client.post(&url).json(&doc).send().await;
            }
            
            start.elapsed()
        });
        
        handles.push(handle);
    }
    
    let mut total_time = std::time::Duration::ZERO;
    for handle in handles {
        total_time += handle.await?;
    }
    
    let ops_per_sec = (ops as f64) / total_time.as_secs_f64();
    println!("{}", format!("Throughput: {:.2} ops/sec", ops_per_sec).green());
    
    Ok(())
}

async fn create_collection(state: &Mutex<AppState>, name: String) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    
    let url = format!("{}/collections", s.base_url());
    let response = s.client.post(&url).json(&serde_json::json!({ "name": name })).send().await?;
    
    if response.status().is_success() {
        println!("{}", format!("Collection '{}' created", name).green());
    } else {
        let error: serde_json::Value = response.json().await?;
        println!("{}", format!("Error: {:?}", error).red());
    }
    
    Ok(())
}

async fn health(state: &Mutex<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let s = state.lock().unwrap();
    
    let url = format!("{}/health", s.base_url());
    let response = s.client.get(&url).send().await?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("{}", serde_json::to_string_pretty(&result)?.green());
    } else {
        println!("{}", "Server unreachable".red());
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let state = Mutex::new(AppState {
        host: cli.host,
        port: cli.port,
        client: Client::new(),
    });
    
    match cli.command {
        Commands::Connect { host, port } => {
            connect(&state, host, port).await?;
        }
        Commands::Insert { collection, doc } => {
            insert(&state, collection, doc).await?;
        }
        Commands::Get { collection, id } => {
            get_doc(&state, collection, id).await?;
        }
        Commands::Query { collection, filter } => {
            query(&state, collection, filter).await?;
        }
        Commands::Vector { collection, field, query, k } => {
            vector_search(&state, collection, field, query, k).await?;
        }
        Commands::Graph { from, hops, label } => {
            graph_traverse(&state, from, hops, label).await?;
        }
        Commands::Bench { ops, threads } => {
            bench(&state, ops, threads).await?;
        }
        Commands::CreateCollection { name } => {
            create_collection(&state, name).await?;
        }
        Commands::Health => {
            health(&state).await?;
        }
    }
    
    Ok(())
}
