use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use futures::future;
use tokio_service::{Service, NewService};
use http;


pub struct FileServer {
    root: PathBuf,
}

impl FileServer {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        FileServer { root: root.into() }
    }
}

impl NewService for FileServer {
    type Request = http::Request;
    type Response = http::Response;
    type Error = io::Error;
    type Instance = FileService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(FileService { root: self.root.clone() })
    }
}


pub struct FileService {
    root: PathBuf,
}

impl Service for FileService {
    type Request = http::Request;
    type Response = http::Response;
    type Error = io::Error;
    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let path = self.root.join(&req.uri[1..]);
        if path.is_file() {
            let mut body = String::new();
            match File::open(path).and_then(|mut f| f.read_to_string(&mut body)) {
                Ok(_) => future::ok(http::Response::with_body(body)),
                Err(err) => future::err(err),
            }
        } else {
            future::err(io::Error::new(io::ErrorKind::Other, "Not found"))
        }
    }
}
