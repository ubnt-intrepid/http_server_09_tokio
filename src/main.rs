extern crate bytes;
extern crate futures;
extern crate num_cpus;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

mod http;
mod service;

use tokio_proto::TcpServer;
use service::FileServer;

fn main() {
    let addr = "0.0.0.0:12345".parse().unwrap();

    let mut server = TcpServer::new(http::Http, addr);
    server.threads(num_cpus::get());

    server.serve(FileServer::new("static/"));
}
