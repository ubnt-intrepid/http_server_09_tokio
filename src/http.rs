use std::io;
use std::str;
use bytes::BytesMut;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_proto::pipeline::ServerProto;


pub struct Request {
    pub uri: String,
}


pub struct Response {
    body: String,
}

impl Response {
    pub fn with_body<B: Into<String>>(body: B) -> Self {
        Response { body: body.into() }
    }
}


pub struct HttpCodec;

impl Decoder for HttpCodec {
    type Item = Request;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        let pos = buf.iter().position(|&b| b == b'\r');
        match pos {
            Some(i) => {
                let line = buf.split_to(i);
                buf.split_to(2);

                if line.starts_with(b"GET ") {
                    match str::from_utf8(&line[4..]) {
                        Ok(s) => Ok(Some(Request { uri: s.to_string() })),
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

impl Encoder for HttpCodec {
    type Item = Response;
    type Error = io::Error;

    fn encode(&mut self, res: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(res.body.as_bytes());
        buf.extend(b"\r\n");
        Ok(())
    }
}


pub struct Http;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for Http {
    type Request = Request;
    type Response = Response;
    type Transport = Framed<T, HttpCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(HttpCodec))
    }
}
