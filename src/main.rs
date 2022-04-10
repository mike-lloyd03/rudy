use std::env;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
mod request;
mod response;
mod tls;
mod utils;
use response::HTTPResponse;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port = if args.len() == 1 {
        8080
    } else {
        args.get(1).unwrap().parse::<i32>().unwrap()
    };
    let addr = format!("127.0.0.1:{}", port);

    let ca_path = "cert/ca.pem";
    let key_path = "cert/key.pem";
    if !Path::new(ca_path).exists() || !Path::new(key_path).exists() {
        println!("Generating CA. This will need to be added to your system/browser trust store for HTTPS requests to be accepted by your browser.");
        let (ca, key) = tls::gen_ca().unwrap();
        tls::cert_to_pem(&ca, ca_path);
        tls::key_to_pem(&key, key_path);
    }
    let root_ca = tls::RootCA::from_pem(ca_path, key_path);

    main_loop(&addr, &root_ca).await;
}

async fn main_loop(addr: &str, root_ca: &tls::RootCA) {
    println!("Rudy is running at {}", addr);
    let mut listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket, &root_ca).await;
        });
    }
}

async fn process(mut socket: TcpStream, root_ca: &tls::RootCA) {
    let http_request = utils::read_http_request(&mut socket).await.unwrap();
    if http_request.path.starts_with("http://") {
        let new_request = http_request.build_request_for_proxy();
        if let Some(resp) = utils::do_request(new_request).await {
            socket.write(&resp.build_message()).await.unwrap();
            println!("Forwarded {}", http_request.path);
        }
    } else if http_request.method == "CONNECT" {
        if let Some(_host) = http_request.get_header_value("Host") {
            let ret = utils::do_connect_request(http_request, &mut socket, &root_ca).await;
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
