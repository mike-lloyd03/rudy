use actix_web::{
    error, middleware,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use awc::Client;
use clap::Arg;
use url::Url;

async fn forward(
    req: HttpRequest,
    payload: web::Payload,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let host = req.headers().get("host").unwrap().to_str().unwrap();
    let mut new_url = Url::parse(&format!("http://{}", host)).unwrap();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());
    println!("req: {:?}\nreq.uri: {:?}", req, req.uri().path());

    // TODO: This forwarded implementation is incomplete as it only handles the inofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = match req.head().peer_addr {
        Some(addr) => forwarded_req.insert_header(("x-forwarded-for", format!("{}", addr.ip()))),
        None => forwarded_req,
    };

    let res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.append_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.streaming(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::Command::new("HTTP Proxy")
        .arg(
            Arg::new("listen_port")
                .takes_value(true)
                .value_name("LISTEN PORT")
                .index(1)
                .required(false),
        )
        .get_matches();

    let listen_addr = "127.0.0.1";
    let listen_port = matches.value_of_t("listen_port").unwrap_or(8080);
    println!("Listening on {}:{}", listen_addr, listen_port);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Client::new()))
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(forward))
    })
    .bind((listen_addr, listen_port))?
    .system_exit()
    .run()
    .await
}
