extern crate bytes;
extern crate futures;
extern crate num_cpus;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

mod http;

use http::{Request, Response, HttpProto};
use std::io;
use futures::future;
use tokio_proto::TcpServer;
use tokio_service::Service;

const BODY: &str = r#"
<html>
<head><title>HTTP 0.9</title></head>
<body><h1>Hello</h1></body>
</html>
"#;

struct HttpService;

impl Service for HttpService {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = future::Ok<Self::Response, Self::Error>;

    fn call(&self, _req: Self::Request) -> Self::Future {
        let res = Response::with_body(BODY);
        future::ok(res)
    }
}

fn main() {
    let addr = "0.0.0.0:12345".parse().unwrap();

    let mut server = TcpServer::new(HttpProto, addr);
    server.threads(num_cpus::get());

    server.serve(|| Ok(HttpService));
}
