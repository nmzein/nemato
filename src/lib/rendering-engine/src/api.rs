mod image;
mod io;
mod db;
mod structs;
mod traits;
mod decoders;

use crate::structs::{AppState, ImageState, ImageSelection};
// use crate::decoders::openslide;

use std::path::PathBuf;
// use std::collections::BTreeMap;
use std::fs;
use std::fmt::Display;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Extension, WebSocketUpgrade
    },
    response::{Json, IntoResponse, Response},
    // http::{Method, StatusCode},
    http::StatusCode,
    routing::{get, post},
    Router, Server
};
use openslide_rs::OpenSlide;
use futures_util::{SinkExt, StreamExt};
use tower_http::{
    cors::{Any, CorsLayer}
};

static IMAGE_NAME: &str = "image-1";
static IMAGE_NAME_EXT: &str = "image-1.tiff";

#[tokio::main]
async fn main() {
    let pool = db::connect().await.unwrap();
    
    let cors: CorsLayer = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any);
    
    let app = Router::new()
        .route("/api/connect", get(connect))
        .route("/api/process", post(process))
        .route("/api/metadata", post(metadata))
        .route("/api/delete", post(delete))
        .layer(cors)
        .layer(Extension(pool));

    Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn connect(socket_upgrader: WebSocketUpgrade, Extension(pool): Extension<AppState>) -> impl IntoResponse {
    socket_upgrader.on_upgrade(|socket| async {
        render(socket, Extension(pool)).await;
    })
}

async fn render(socket: WebSocket, Extension(pool): Extension<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(Ok(Message::Text(message))) = receiver.next().await {
        if let Ok(selection) = serde_json::from_str::<ImageSelection>(&message) {
            // ** Example Only ** //
            let level = 0;
            // ****************** //
            
            println!("Received selection: {:?}", selection);


            if let Ok(Some(image)) = db::get(IMAGE_NAME, &pool).await {
                match io::retrieve(
                    &image.store_path,
                    &level,
                    &selection,
                ) {
                    Ok(tiles) => {
                        println!("Sending {} tiles.", tiles.len());
                        for tile in tiles {
                            let _ = sender.send(Message::Binary(tile)).await.map_err(|err| {
                                eprintln!("Error sending tile: {}", err);
                            });
                        }
                    }
                    Err(err) => {
                        eprintln!("Error reading from datastore: {}", err);
                    }
                }
            } else {
                // Not only error.
                eprintln!("ImageState with name {} does not exist.", IMAGE_NAME);
            }

        } else {
            eprintln!("Error deserialising selection.");
        }
    }
}

async fn metadata(Extension(pool): Extension<AppState>) -> Response {
    match db::get(IMAGE_NAME, &pool).await {
        Ok(Some(image)) => {
            Json(image.metadata).into_response()
        },
        Ok(None) => {
            log_respond::<String>(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Metadata for image with name {} does not exist.", IMAGE_NAME).as_str(),
                None
            )
        },
        Err(err) => {
            log_respond(StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve metadata.", Some(err))
        }
    }
}

async fn process(Extension(pool): Extension<AppState>) -> Response {
    // Strip file extension.
    let id = IMAGE_NAME_EXT.split('.').collect::<Vec<&str>>()[0];
    println!("Processing image with name: {}", id);

    if db::contains(&id, &pool).await {
        return log_respond::<String>(
            StatusCode::BAD_REQUEST,
            format!("ImageState with name {} already exists. Consider deleting it from the list first.", id).as_str(),
            None
        );
    }

    let image_path = PathBuf::from(format!("store/{}/{}", id, IMAGE_NAME_EXT));
    let store_path = PathBuf::from(format!("store/{}/{}.zarr", id, id));

    match io::convert::<OpenSlide>(
        &image_path,
        &store_path,
    ) {
        Ok(metadata) => {
            // TODO: Error handling.
            let _ = db::insert(id, &ImageState { image_path, store_path, metadata }, &pool).await;
            log_respond::<String>(StatusCode::OK, "Successfully processed image.", None)
        },
        Err(err) => log_respond(StatusCode::INTERNAL_SERVER_ERROR, "Failed to process the image.", Some(err))
    }
}

async fn delete(Extension(pool): Extension<AppState>) -> Response {
    // TODO: Move to IO.
    let dir_path = PathBuf::from("store/".to_owned() + IMAGE_NAME);
    
    // Remove directory.
    let _  = fs::remove_dir_all(dir_path).map_err(|err| {
        return log_respond(StatusCode::INTERNAL_SERVER_ERROR, "Could not delete directory.", Some(err));
    });
    
    // TODO: Error handling.
    let _ = db::remove(IMAGE_NAME, &pool).await;

    log_respond::<String>(StatusCode::OK, "Successfully deleted image entry.", None)
}

fn log_respond<T: Display>(status_code: StatusCode, message: &str, details: Option<T>) -> Response {
    if status_code.is_success() {
        println!("Ok: {}", message);
        if let Some(details) = details {
            println!("Details: {}", details);
        }
    } else {
        eprintln!("Error <{}>: {}", status_code, message);
        if let Some(details) = details {
            eprintln!("Details: {}", details);
        }
    }

    (status_code, String::from(message)).into_response()
}