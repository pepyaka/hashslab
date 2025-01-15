use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hashslab::HashSlabMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Endpoint {
    url: String,
    state: EndpointState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum EndpointState {
    Connected,
    Disconnected,
}

#[derive(Debug, Serialize, Deserialize)]
struct WithId<T> {
    id: usize,
    #[serde(flatten)]
    data: T,
}

type Endpoints = Arc<Mutex<HashSlabMap<String, EndpointState>>>;

async fn list_endpoints(State(endpoints): State<Endpoints>) -> Json<Vec<WithId<Endpoint>>> {
    let endpoints = {
        let lock = endpoints.lock().unwrap();
        lock.iter_full()
            .map(|(id, url, state)| WithId {
                id,
                data: Endpoint {
                    url: url.clone(),
                    state: *state,
                },
            })
            .collect::<Vec<_>>()
    };
    Json(endpoints)
}

async fn add_endpoint(
    State(endpoints): State<Endpoints>,
    Json(endpoint): Json<Endpoint>,
) -> Json<WithId<Endpoint>> {
    let id = {
        let mut lock = endpoints.lock().unwrap();
        let (idx, _) = lock.insert_full(endpoint.url.clone(), endpoint.state);
        idx
    };
    Json(WithId { id, data: endpoint })
}

async fn get_endpoint(
    Path(id): Path<usize>,
    State(endpoints): State<Endpoints>,
) -> impl IntoResponse {
    let endpoint = {
        let lock = endpoints.lock().unwrap();
        lock.get_index(id).map(|(url, &state)| Endpoint {
            url: url.clone(),
            state,
        })
    };
    if let Some(endpoint) = endpoint {
        (StatusCode::OK, Ok(Json(endpoint)))
    } else {
        (StatusCode::NOT_FOUND, Err(Json("Not found")))
    }
}

async fn remove_endpoint(
    Path(id): Path<usize>,
    State(endpoints): State<Endpoints>,
) -> impl IntoResponse {
    let endpoint = {
        let mut lock = endpoints.lock().unwrap();
        lock.remove_index(id).map(|(url, state)| Endpoint {
            url: url.clone(),
            state,
        })
    };
    if let Some(endpoint) = endpoint {
        (StatusCode::OK, Ok(Json(endpoint)))
    } else {
        (StatusCode::NOT_FOUND, Err(Json("Not found")))
    }
}

#[tokio::main]
async fn main() {
    let endpoints = Arc::new(Mutex::new(HashSlabMap::new()));

    let app = Router::new()
        .route("/endpoints", get(list_endpoints).post(add_endpoint))
        .route("/endpoints/{id}", get(get_endpoint).delete(remove_endpoint))
        .with_state(endpoints);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Running on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
