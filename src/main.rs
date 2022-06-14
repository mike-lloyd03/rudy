use hudsucker::ProxyBuilder;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::error;

mod proxy;
mod rudy_tui;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (tx, mut rx) = mpsc::channel(16);
    let listen_port = 8080;

    let app = rudy_tui::App::new(&mut rx);

    let ca = proxy::load_ca("cert/ca.crt", "cert/ca.key");

    let proxy = ProxyBuilder::new()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], listen_port)))
        .with_rustls_client()
        .with_ca(ca)
        .with_http_handler(proxy::LogHandler { tx })
        .build();

    // println!("Now listening on 127.0.0.1:{}", listen_port);

    tokio::spawn(async move { proxy.start(shutdown_signal()).await });

    // if let Err(e) = proxy.start(shutdown_signal()).await {
    //     error!("{}", e);
    // }
    rudy_tui::run(app).await.unwrap();
}
