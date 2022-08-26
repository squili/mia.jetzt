use std::{convert::Infallible, str::FromStr, sync::Arc};

use config::{Action, Config};
use hyper::{
    client::HttpConnector,
    header::Entry,
    http::HeaderValue,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Client, HeaderMap, Request, Response, Server, Uri,
};
use tracing::error;

mod config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Arc::new(config::parse()?);
    let client = Client::new();
    let make_svc = make_service_fn(|_: &AddrStream| {
        let client = client.clone();
        let config = config.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |request| {
                error_handler(request, config.clone(), client.clone())
            }))
        }
    });
    let server = Server::bind(&config.listen_on).serve(make_svc);

    Ok(server.await?)
}

async fn error_handler(
    request: Request<Body>,
    config: Arc<Config>,
    http_client: Client<HttpConnector>,
) -> Result<Response<Body>, Infallible> {
    let response = request_handler(request, config, http_client)
        .await
        .unwrap_or_else(|err| {
            error!("Error in request handler: {err:?}");

            Response::builder()
                .header("Content-Type", "text/html")
                .status(500)
                .body("<h1>500 Internal Server Error</h1><pre>Router</pre>".into())
                .unwrap()
        });
    Ok(response)
}

const HOP_HEADERS: &[&str] = &[
    "Connection",
    "Keep-Alive",
    "Proxy-Authenticate",
    "Proxy-Authorization",
    "Te",
    "Trailers",
    "Transfer-Encoding",
    "Upgrade",
];

fn strip_hop_headers(headers: &mut HeaderMap<HeaderValue>) {
    for header in HOP_HEADERS {
        let entry = headers.entry(*header);
        if let Entry::Occupied(entry) = entry {
            entry.remove_entry_mult();
        }
    }
}

async fn request_handler(
    mut request: Request<Body>,
    config: Arc<Config>,
    http_client: Client<HttpConnector>,
) -> eyre::Result<Response<Body>> {
    let host = match request.headers().get("host") {
        Some(host) => String::from_utf8_lossy(host.as_bytes()).to_string(),
        None => {
            return Ok(Response::builder()
                .status(400)
                .body("Missing host header".into())?)
        }
    };

    for route in &config.routes {
        if route.host == host {
            match &route.action {
                Action::Forward { target } => {
                    strip_hop_headers(request.headers_mut());
                    let mut built_uri = format!("{target}{}", request.uri().path());
                    if let Some(query) = request.uri().query() {
                        built_uri.push('?');
                        built_uri.push_str(query);
                    }
                    *request.uri_mut() = Uri::from_str(&built_uri)?;
                    let mut response = http_client.request(request).await?;
                    strip_hop_headers(response.headers_mut());
                    return Ok(response);
                }
                Action::Redirect { target, permanent } => {
                    return Ok(Response::builder()
                        .header("Location", target)
                        .status(if *permanent { 308 } else { 307 })
                        .body(Body::empty())?)
                }
            }
        }
    }

    Ok(Response::builder()
        .header("Content-Type", "text/html")
        .status(404)
        .body("<h1>404 Unknown Host</h1><pre>Router</pre>".into())?)
}
