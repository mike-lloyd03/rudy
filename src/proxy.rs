use hudsucker::{
    async_trait::async_trait,
    certificate_authority::RcgenAuthority,
    hyper::{body::to_bytes, Body, Request, Response},
    *,
};
use rustls_pemfile as pemfile;
use std::process::exit;
use std::str;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct LogHandler {
    pub tx: Sender<String>,
}

#[async_trait]
impl HttpHandler for LogHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        self.tx.send("Sending message".to_string()).await.unwrap();
        RequestOrResponse::Request(format_req(req).await)
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        // println!("{:?}", res);
        res
    }
}

async fn format_req(req: Request<Body>) -> Request<Body> {
    let new_req: Request<Body>;
    let (parts, body) = req.into_parts();

    let mut output = "----------\n".to_string();
    output += &format!("{} {} {:?}\n", parts.method, parts.uri, parts.version);
    for h in &parts.headers {
        output += &format!("{}: {:?}\n", h.0, h.1)
    }

    if parts.headers.contains_key("Content-Length")
        || parts.headers.contains_key("Transfer-Encoding")
    {
        let body_bytes = to_bytes(body).await.unwrap();
        output += "body:\n";
        output += str::from_utf8(&body_bytes).unwrap();

        let new_body = Body::from(body_bytes);
        new_req = Request::from_parts(parts, new_body);
    } else {
        new_req = Request::from_parts(parts, body);
    }

    // println!("{}", output);

    new_req
}

/// Loads the certificate authority and private key for the proxy server.
pub fn load_ca(cert_path: &str, key_path: &str) -> RcgenAuthority {
    let ca_cert_bytes = match std::fs::read_to_string(cert_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Unable to load cert file at '{}'. {}", cert_path, e);
            exit(1)
        }
    };

    let private_key_bytes = match std::fs::read_to_string(key_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Unable to load key file at '{}'. {}", key_path, e);
            exit(1)
        }
    };

    let ca_cert = rustls::Certificate(
        pemfile::certs(&mut ca_cert_bytes.as_bytes())
            .expect("Failed to parse CA certificate")
            .remove(0),
    );

    let private_key = rustls::PrivateKey(
        pemfile::pkcs8_private_keys(&mut private_key_bytes.as_bytes())
            .expect("Failed to parse private key")
            .remove(0),
    );

    RcgenAuthority::new(private_key, ca_cert, 1_000)
        .expect("Failed to create Certificate Authority")
}
