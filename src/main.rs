use std::env;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
mod request;
mod response;
mod tls;
mod utils;
use response::HTTPResponse;

const CA_PATH: &str = "cert/ca.pem";
const KEY_PATH: &str = "cert/key.pem";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port = if args.len() == 1 {
        8080
    } else {
        args.get(1).unwrap().parse::<i32>().unwrap()
    };
    let addr = format!("127.0.0.1:{}", port);

    if !Path::new(CA_PATH).exists() || !Path::new(KEY_PATH).exists() {
        println!("Generating CA. This will need to be added to your system/browser trust store for HTTPS requests to be accepted by your browser.");
        let (ca, key) = tls::gen_ca().unwrap();
        tls::cert_to_pem(&ca, CA_PATH).unwrap();
        tls::key_to_pem(&key, KEY_PATH).unwrap();
    }

    main_loop(&addr).await;
}

async fn main_loop(addr: &str) {
    println!("Rudy is running at {}", addr);
    let mut listener = TcpListener::bind(addr).await.unwrap();
    let mut cert_cache: tls::CertCache =
        tls::CertCache::new(tls::RootCA::from_pem(CA_PATH, KEY_PATH));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket, &mut cert_cache).await;
        });
    }
}

async fn process(mut socket: TcpStream, cert_cache: &mut tls::CertCache) {
    let http_request = utils::read_http_request(&mut socket).await.unwrap();
    if http_request.path.starts_with("http://") {
        let new_request = http_request.build_request_for_proxy();
        if let Some(resp) = utils::do_request(new_request).await {
            socket.write(&resp.build_message()).await.unwrap();
            println!("Forwarded {}", http_request.path);
        }
    } else if http_request.method == "CONNECT" {
        if let Some(_host) = http_request.get_header_value("Host") {
            let ret = utils::do_connect_request(http_request, &mut socket).await;
            match ret {
                Some(addr) => {
                    println!("Forwarded {}", addr);
                }
                None => {
                    println!("An unknown HTTPS request");
                }
            }
        }
    } else {
        println!("Unknown request: {:?}", http_request);
        send_501_error(&mut socket).await;
    }
}

async fn send_501_error(socket: &mut TcpStream) {
    let http_response_content = HTTPResponse::create_501_error().build_message();
    if let Err(err) = socket.write(&http_response_content).await {
        panic!("{}", err);
    }
}
