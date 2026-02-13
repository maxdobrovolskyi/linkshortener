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


type SharedState = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {

    let links: SharedState = Arc::new(Mutex::new(HashMap::new()));
    tracing_subscriber::fmt::init();
    
    let app = Router::new().route("/", get(root))
        .route("/shorter", get(shorter))
        .route("/{*path}", get(give_link))
        .with_state(links); 
    
    let  listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root () -> String {
    tracing::info!("Hello");
    "Hello World!".to_string()
}

async fn give_link(
        Path(path): Path<String>,
        State(links): State<SharedState>
    ) -> impl IntoResponse {
    let map = links.lock().unwrap();

    tracing::info!("{path}");
    for (full_path, short) in map.iter() {
            println!("{short}, {full_path}, {path}");
            if path.to_string() == short.to_string() {
                println!("Yes");
                return Redirect::permanent(full_path).into_response();         
            }
    }

    (axum::http::StatusCode::NOT_FOUND, "Link not found").into_response()
}


async fn shorter (
    State(links): State<SharedState>,
    Query(params): Query<HashMap<String, String>>
    ) -> String {
    tracing::info!("Sorter page");
    
    
    let url_to_shorter = params.get("url");
    
    match url_to_shorter {
        Some(target_url) => {

            let root_url = "http://localhost:3000/";
            let rundom_slug = Alphanumeric.sample_string(&mut rand::rng(), 7);
            //let shorter_url = format!("{}{}", root_url, rundom_slug);
            let shorter_url = format!("{rundom_slug}");
            let mut map = links.lock().unwrap();
            {
                match map.get(target_url) {
                    Some(url) => {
                        tracing::info!("Shorted {} to {}", target_url, url);
                        format!("Shorted link => {}", url)         
                    }
                    None => {
                        map.insert(target_url.to_string(), shorter_url.to_string());
                        tracing::info!("Shorted {} to {}", target_url, shorter_url);
                        format!("Shorted link => {}", shorter_url)
                    } 
                }
            }
        },
        None => {
           return  "Error".to_string();
        }
    }

}

