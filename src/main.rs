use std::{convert::Infallible, net::SocketAddr};

use http_body_util::BodyExt;
use hyper::{body::Body, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), String> {
    // determine listen address
    let addr = match std::env::var("LISTEN_ADDR") {
        Ok(addr) => addr.parse().expect("LISTEN ADDR"),
        Err(_) => SocketAddr::from(([127, 0, 0, 1], 8080)),
    };

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| format!("Failed to bind to {addr}: {err}"))?;

    println!("===> Listening on http://{addr}");

    let server = run_server(listener);
    let shutdown = shutdown_signal();

    tokio::select! {
        res = server => res,
        _ = shutdown => Ok(())
    }
}

/// Run the web server
async fn run_server(listener: TcpListener) -> Result<(), String> {
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|err| format!("Failed to accept connection: {err}"))?;

        tokio::task::spawn(async move {
            // convert to something Tokio understands
            let io = TokioIo::new(stream);
            // call handler
            let res = http1::Builder::new()
                .serve_connection(io, service_fn(inspect))
                .await;
            // handle errors
            if let Err(err) = res {
                eprintln!("Error serving connection: {err}");
            }
        });
    }
}

/// Handler method
async fn inspect(req: Request<hyper::body::Incoming>) -> Result<Response<String>, Infallible> {
    // Print headers
    let headers = req
        .headers()
        .into_iter()
        .map(|(key, value)| {
            let value = value
                .to_str()
                .map(String::from)
                .unwrap_or_else(|_| format!("{value:?}"));
            format!("{key}: {value:?}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Generate request & headers line
    // we do this here since we consume the request below
    let out = format!(
        "{} {} {:?}\nquery: {}\n\n{}",
        req.method(),
        req.uri().path(),
        req.version(),
        req.uri().query().unwrap_or_default(),
        headers,
    );

    // Ensure we don't get nuked by huge bodies
    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if upper > 64 * 1024 {
        let mut res = Response::new(format!("Body too large: {upper}"));
        *res.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(res);
    }

    // buffer the whole body
    let body = match req.collect().await {
        Ok(body) => body.to_bytes(),
        Err(err) => {
            let mut res = Response::new(format!("Failed to read body: {err}"));
            *res.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(res);
        }
    };

    // Try to conver the body to a string, just print its length otherwise.
    let body = match body {
        _ if body.is_empty() => String::from("[no body]"),
        body if body.is_ascii() => {
            String::from_utf8(body.to_vec()).expect("convert ASCII-only body")
        }
        body => format!("[{} bytes of body data]", body.len()),
    };

    let out = format!("{out}\n\n{body}");
    Ok(Response::new(out))
}

/// This method will block (async) until either SIGTERM or SIGINT have been sent to the process.
async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let ctrl_c = async {
        signal(SignalKind::interrupt())
            .expect("Failed to install SIGINT handler")
            .recv()
            .await
    };

    let terminate = async {
        signal(SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
