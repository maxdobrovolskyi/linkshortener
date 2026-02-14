use tokio;
use axum::{
    routing::get,
    Router,
    extract::Query,
    extract::State,
};
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::http::StatusCode;
use anyhow::{Context, Result};


type SharedState = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() -> Result<()>{

    let links: SharedState = Arc::new(Mutex::new(HashMap::new()));
    tracing_subscriber::fmt::init();
    
    let app = Router::new().route("/", get(root))
        .route("/shorter", get(shorter))
        .route("/{*path}", get(give_link))
        .with_state(links); 
    
    let  listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.context("Can not use this port")?;
    axum::serve(listener, app).await.context("Error start server")?;

    Ok(())
}

async fn root () -> String {
    tracing::info!("Hello");
    "Hello World!".to_string()
}

async fn give_link(
        Path(path): Path<String>,
        State(links): State<SharedState>
    ) -> impl IntoResponse {
    let map = match links.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::error!("Mutex is poisoned");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
        }
    };

    tracing::info!("{path}");
    for (full_path, short) in map.iter() {
            println!("{short}, {full_path}, {path}");
            if path.to_string() == short.to_string() {
                println!("Yes");
                return Redirect::permanent(full_path).into_response();         
            }
    }

    (StatusCode::NOT_FOUND, "Link not found").into_response()
}


async fn shorter (
    State(links): State<SharedState>,
    Query(params): Query<HashMap<String, String>>
    ) -> impl IntoResponse {
    tracing::info!("Sorter page");
    
    
    let url_to_shorter = params.get("url");
    
    match url_to_shorter {
        Some(target_url) => {

            //let root_url = "http://localhost:3000/";
            let rundom_slug = Alphanumeric.sample_string(&mut rand::rng(), 7);
            //let shorter_url = format!("{}{}", root_url, rundom_slug);
            let shorter_url = format!("{rundom_slug}");
            let mut map = match links.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    tracing::error!("Mutex is poisoned");
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
                }
            };
            {
                match map.get(target_url) {
                    Some(url) => {
                        tracing::info!("Shorted {} to {}", target_url, url);
                        format!("Shorted link => {}", url).into_response()         
                    }
                    None => {
                        map.insert(target_url.to_string(), shorter_url.to_string());
                        tracing::info!("Shorted {} to {}", target_url, shorter_url);
                        format!("Shorted link => {}", shorter_url).into_response()
                    } 
                }
            }
        },
        None => {
           return  "Error".into_response();
        }
    }

}

