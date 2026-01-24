use std::{io::{self, Read, Result, Write}, net::TcpStream};
use ffi::Event;
use poll::Poll;
mod ffi;
mod poll;

struct HandlerResponse {
    completed_count: usize,
    handled_idx: Vec<usize>,
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}

// fn handle_events(events: &[Event], streams: &mut [TcpStream]) -> Result<usize> {
fn handle_events(events: &[Event], streams: &mut [TcpStream]) -> Result<HandlerResponse> {
    let mut handled_idx = vec![];
    let mut handled_events = 0;
    for event in events {
        let index = event.token();
        let mut data = vec![0u8; 4096];
        loop {
            match streams[index].read(&mut data) {
                Ok(n) if n == 0 => {
                    handled_events += 1;
                    handled_idx.push(index);
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    println!("RECEIVED: {:?}", event);
                    println!("{txt}\n------\n");
                }
                // Not ready to read in a non-blocking manner. This could
                // happen even if the event was reported as ready -> WHY ??
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
    }
    Ok(HandlerResponse { completed_count: handled_events, handled_idx })
}

fn main() -> Result<()> {
    let mut poll = Poll::new()?;
    let n_events = 5;
    let mut streams = vec![];
    let addr = "localhost:8080";
    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let mut stream = std::net::TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        stream.write_all(request.as_bytes())?;
        poll.registry()
        .register(&stream, i, ffi::EPOLLIN | ffi::EPOLLET)?;
        streams.push(stream);
    }
    let mut handled_events = 0;
    while handled_events < n_events {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;
        if events.is_empty() {
            println!("TIMEOUT (OR SPURIOUS EVENT NOTIFICATION)");
            continue;
        }
        let resp = handle_events(&events, &mut streams)?;
        handled_events += resp.completed_count;
        poll.registry().deregister(&streams[resp.handled_idx[0]])?;
    }
    println!("FINISHED");
    Ok(())
}
