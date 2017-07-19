extern crate bytes;
extern crate futures;
extern crate num_cpus;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

pub struct HttpRequest(String);
pub struct HttpResponse(String);

/// Step1: Implement a codec
mod codec {
    use std::io;
    use std::str;
    use tokio_io::codec::{Encoder, Decoder};
    use bytes::BytesMut;
    use super::{HttpRequest, HttpResponse};

    pub struct LineCodec;

    impl Encoder for LineCodec {
        type Item = HttpResponse;
        type Error = io::Error;

        fn encode(&mut self, res: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
            let HttpResponse(ref body) = res;
            buf.extend(body.as_bytes());
            buf.extend(b"\r\n");
            Ok(())
        }
    }

    impl Decoder for LineCodec {
        type Item = HttpRequest;
        type Error = io::Error;

        fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
            let pos = buf.iter().position(|&b| b == b'\n');
            match pos {
                Some(i) => {
                    // remove the serialized frame from the buffer.
                    let line = buf.split_to(i);
                    // Also remove the '\n';
                    buf.split_to(1);

                    if line.starts_with(b"GET ") {
                        // Turn this data into a UTF8 string and return it in a Frame.
                        match str::from_utf8(&line[4..]) {
                            Ok(s) => Ok(Some(HttpRequest(s.trim_right_matches("\r").to_string()))),
                            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
                        }
                    } else {
                        Err(io::Error::new(io::ErrorKind::Other, "invalid protocol"))
                    }
                }
                None => Ok(None),
            }
        }
    }
}

// Step2: Specify the protocol
mod proto {
    use tokio_proto::pipeline::ServerProto;
    use tokio_io::{AsyncRead, AsyncWrite};
    use tokio_io::codec::Framed;
    use std::io;
    use super::{HttpRequest, HttpResponse};
    use super::codec::LineCodec;

    pub struct LineProto;

    impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
        type Request = HttpRequest;
        type Response = HttpResponse;
        type Transport = Framed<T, LineCodec>;
        type BindTransport = Result<Self::Transport, io::Error>;

        fn bind_transport(&self, io: T) -> Self::BindTransport {
            Ok(io.framed(LineCodec))
        }
    }
}

mod service {
    use tokio_service::Service;
    use futures::{future, Future, BoxFuture};
    use std::io;
    use super::{HttpRequest, HttpResponse};

    pub struct HttpService;

    impl Service for HttpService {
        type Request = HttpRequest;
        type Response = HttpResponse;
        type Error = io::Error;
        type Future = BoxFuture<Self::Response, Self::Error>;

        fn call(&self, req: Self::Request) -> Self::Future {
            let HttpRequest(uri) = req;
            let res = HttpResponse(format!(
                r#"
<html>
<head><title>HTTP 0.9</title></head>
<body><h1>Hello</h1></body>
</html>
"#,
            ));
            future::ok(res).boxed()
        }
    }
}

fn main() {
    use tokio_proto::TcpServer;
    use proto::LineProto;
    use service::HttpService;

    let addr = "0.0.0.0:12345".parse().unwrap();

    let mut server = TcpServer::new(LineProto, addr);
    server.threads(num_cpus::get());

    server.serve(|| Ok(HttpService));
}
