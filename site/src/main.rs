use std::{env, sync::Arc};

use axum::{
    body::Body,
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use include_dir::{include_dir, Dir};
use tera::{Context, Tera};
use tracing::error;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let tera = Tera::new("templates/**/*.html")?;

    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(|| async { "NERD" }))
        .route("/assets/:name", get(assets))
        .layer(Extension(Arc::new(tera)));

    let listen_on = env::var("LISTEN_ON")?.parse()?;
    println!("Listening on {listen_on}");
    if let Err(err) = axum::Server::bind(&listen_on)
        .serve(app.into_make_service())
        .await
    {
        error!(%err, "error running axum server");
    }

    Ok(())
}

fn render(tera: Arc<Tera>, name: &str, context: &Context) -> impl IntoResponse {
    match tera.render(name, context) {
        Ok(text) => Response::builder()
            .header("Content-Type", "text/html")
            .body(Body::from(text))
            .unwrap(),
        Err(err) => Response::builder()
            .body(Body::from(format!("Error rendering template: {err:?}")))
            .unwrap(),
    }
}

async fn index(Extension(tera): Extension<Arc<Tera>>) -> impl IntoResponse {
    render(tera, "index.html", &Context::new())
}

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

async fn assets(Path(name): Path<String>) -> impl IntoResponse {
    match ASSETS_DIR.get_file(name) {
        Some(file) => Response::builder()
            .body(Body::from(file.contents()))
            .unwrap(),
        None => Response::builder()
            .status(404)
            .body("File not found".into())
            .unwrap(),
    }
}
