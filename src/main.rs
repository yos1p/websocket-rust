mod messages;
mod websocket;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Result, Server, StatusCode};
use std::net::SocketAddr;

/// Responsible for routing match
async fn server_kernel(req: Request<Body>) -> Result<Response<Body>> {
    let mut resp = Response::new(Body::from("Hello Rust!"));

    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/messages") => {
            let msg_handler = messages::MessageHandler();
            return websocket::upgrade_connection(req, msg_handler).await;
        }
        (&hyper::Method::GET, "/version") => {
            let version_handler = websocket::VersionHandler();
            return websocket::upgrade_connection(req, version_handler).await;
        }
        _ => {
            *resp.status_mut() = StatusCode::OK;
        }
    }

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    let make_svc = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(server_kernel)) });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on http://{}", addr);
    server.await?;

    Ok(())
}
