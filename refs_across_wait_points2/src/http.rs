use crate::{
    Future,
    future::PollState,
    runtime::{self, Waker, reactor},
};
use mio::Interest;
use std::io::{ErrorKind, Read, Write};
use std::pin::Pin;

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
    Host: localhost\r\n\
    Connection: close\r\n\
    \r\n"
    )
}

pub struct Http;
impl Http {
    pub fn get(path: String) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
    id: usize,
}

impl HttpGetFuture {
    fn new(path: String) -> Self {
        Self {
            stream: None,
            buffer: vec![],
            path: path.to_string(),
            id: reactor().next_id(),
        }
    }

    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap(); // confirm!
        let mut stream = mio::net::TcpStream::from_std(stream);
        stream.write_all(get_req(&self.path).as_bytes()).unwrap();
        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = String;
    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> PollState<Self::Output> {
        let id = self.id;
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            self.write_request();
            let stream = (&mut self).stream.as_mut().unwrap();
            runtime::reactor().register(stream, Interest::READABLE, id);
            runtime::reactor().set_waker(waker, self.id);
        }
        let mut buff = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer).to_string();
                    runtime::reactor().deregister(self.stream.as_mut().unwrap(), id);
                    break PollState::Ready(s);
                }
                Ok(n) => {
                    self.buffer.extend(&buff[0..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    runtime::reactor().set_waker(waker, self.id);
                    break PollState::NotReady;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}
