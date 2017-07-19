use std::io;
use std::str;
use bytes::BytesMut;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_proto::pipeline::ServerProto;


pub struct Request(String);

impl Request {
    fn decode(buf: &mut BytesMut) -> io::Result<Option<Self>> {
        let pos = buf.iter().position(|&b| b == b'\r');
        match pos {
            Some(i) => {
                // remove the serialized frame from the buffer.
                let line = buf.split_to(i);
                // Also remove the '\r\n';
                buf.split_to(2);

                if line.starts_with(b"GET ") {
                    // Turn this data into a UTF8 string and return it in a Frame.
                    match str::from_utf8(&line[4..]) {
                        Ok(s) => Ok(Some(Request(s.to_string()))),
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


pub struct Response(String);

impl Response {
    pub fn with_body<S: Into<String>>(s: S) -> Self {
        Response(s.into())
    }
    fn encode(&self, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(self.0.as_bytes());
        buf.extend(b"\r\n");
        Ok(())
    }
}


pub struct HttpCodec;

impl Decoder for HttpCodec {
    type Item = Request;
    type Error = io::Error;
    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        Request::decode(buf)
    }
}

impl Encoder for HttpCodec {
    type Item = Response;
    type Error = io::Error;
    fn encode(&mut self, res: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        res.encode(buf)
    }
}


pub struct HttpProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for HttpProto {
    type Request = Request;
    type Response = Response;
    type Transport = Framed<T, HttpCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(HttpCodec))
    }
}
