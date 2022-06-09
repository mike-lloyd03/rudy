use hudsucker::{
    async_trait::async_trait,
    certificate_authority::RcgenAuthority,
    hyper::{Body, Request, Response},
    *,
};
use rustls_pemfile as pemfile;
use std::net::SocketAddr;
use tracing::*;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[derive(Clone)]
struct LogHandler;

#[async_trait]
impl HttpHandler for LogHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        let d_req = decode_request(req).unwrap();
        format_req(&d_req);
        RequestOrResponse::Request(d_req)
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        // println!("{:?}", res);
        res
    }
}

fn format_req(req: &Request<Body>) {
    println!("--------");
    println!("{} {} {:?}", req.method(), req.uri(), req.version(),);
    for h in req.headers() {
        println!("{}: {:?}", h.0, h.1)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let listen_port = 8080;

    let mut private_key_bytes: &[u8] = include_bytes!("../ca/hudsucker.key");
    let mut ca_cert_bytes: &[u8] = include_bytes!("../ca/hudsucker.cer");
    let private_key = rustls::PrivateKey(
        pemfile::pkcs8_private_keys(&mut private_key_bytes)
            .expect("Failed to parse private key")
            .remove(0),
    );
    let ca_cert = rustls::Certificate(
        pemfile::certs(&mut ca_cert_bytes)
            .expect("Failed to parse CA certificate")
            .remove(0),
    );

    let ca = RcgenAuthority::new(private_key, ca_cert, 1_000)
        .expect("Failed to create Certificate Authority");

    let proxy = ProxyBuilder::new()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], listen_port)))
        .with_rustls_client()
        .with_ca(ca)
        .with_http_handler(LogHandler)
        // .with_websocket_handler(WsLogHandler)
        .build();

    println!("Now listening on 127.0.0.1:{}", listen_port);

    if let Err(e) = proxy.start(shutdown_signal()).await {
        error!("{}", e);
    }
}
