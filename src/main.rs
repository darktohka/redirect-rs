use http_body_util::Full;
use hyper::body::{Bytes, Incoming}; // Corrected: Import Full from hyper::body
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use regex_lite::Regex;
use socket2::{Domain, Socket, Type};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug)]
struct RedirectRule {
    from: Regex,
    to: String,
}

fn parse_redirect_rules() -> Vec<RedirectRule> {
    let mut rules = Vec::new();
    let mut env_vars: HashMap<String, String> = env::vars().collect();

    let mut keys_to_remove = Vec::new();

    for (key, value) in &env_vars {
        if key.starts_with("REDIRECT_") && key.ends_with("_FROM") {
            let base_name = key
                .trim_start_matches("REDIRECT_")
                .trim_end_matches("_FROM");
            let to_key = format!("REDIRECT_{}_TO", base_name);

            if let Some(to_value) = env_vars.get(&to_key) {
                match Regex::new(value) {
                    Ok(regex) => {
                        rules.push(RedirectRule {
                            from: regex,
                            to: to_value.clone(),
                        });
                        keys_to_remove.push(key.clone());
                        keys_to_remove.push(to_key.clone());
                    }
                    Err(e) => {
                        eprintln!("Error compiling regex for {}: {}", key, e);
                    }
                }
            } else {
                eprintln!(
                    "Warning: Found {} but no matching {} environment variable.",
                    key, to_key
                );
            }
        }
    }

    // Remove processed environment variables to avoid re-processing or warnings for unmatched TOs
    for key in keys_to_remove {
        env_vars.remove(&key);
    }

    rules
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rules = parse_redirect_rules();
    for rule in &rules {
        println!("Redirect Rule: {:?} -> {}", rule.from, rule.to);
    }

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port);

    let socket = Socket::new(Domain::IPV6, Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    socket.set_only_v6(false)?;
    socket.bind(&addr.into())?;
    socket.listen(1024)?;
    socket.set_nonblocking(true)?; // Set socket to non-blocking mode

    let listener = TcpListener::from_std(socket.into())?;

    let redirect_rules = Arc::new(rules);

    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let redirect_rules = Arc::clone(&redirect_rules);
        let ip_address = io
            .inner()
            .peer_addr()
            .ok()
            .map(|addr| addr.ip().to_string());

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        handle_request(req, ip_address.clone(), Arc::clone(&redirect_rules))
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<Incoming>,
    original_ip_address: Option<String>,
    rules: Arc<Vec<RedirectRule>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let path = format!("{}{}", host, req.uri());
    let fallback_ip = original_ip_address.unwrap_or("unknown".to_string());
    let ip_address = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or(fallback_ip.as_ref());

    for rule in rules.iter() {
        if rule.from.is_match(&path) {
            let redirected_path = rule.from.replace_all(&path, &rule.to[..]);
            let response = Response::builder()
                .status(StatusCode::FOUND)
                .header(hyper::header::LOCATION, redirected_path.as_ref())
                .body(Full::new(Bytes::new()))
                .unwrap();
            println!("[{}] {} -> {}", ip_address, path, redirected_path);
            return Ok(response);
        }
    }

    println!("[{}] {} -> 404", ip_address, path);
    let mut not_found = Response::new(Full::new(Bytes::from("Not Found")));
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    Ok(not_found)
}
