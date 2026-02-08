use crate::future::{Future, PollState};
use mio::{Events, Poll, Registry};
use std::{sync::OnceLock, time::Instant};

static REGISTRY: OnceLock<Registry> = OnceLock::new();
pub fn registry() -> &'static Registry {
    REGISTRY.get().expect("Called outside a runtime context")
}

pub struct Runtime {
    poll: Poll,
}

impl Runtime {
    pub fn new() -> Self {
        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        REGISTRY.set(registry).unwrap();
        Self { poll }
    }

    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String>,
    {
        let mut future = future;
        loop {
            match future.poll() {
                PollState::NotReady => {
                    println!("Schedule other tasks\n");
                    let mut events = Events::with_capacity(100);
                    let x = Instant::now();
                    self.poll.poll(&mut events, None).unwrap();
                    // the following proves that execution does not move ahead when waiting for event
                    // this snippet just proves that we have a way to immediately wake up when an event becomes read/write-able.
                    // BUT efficient usage of CPU time is still pending
                    println!("immediately after polling : {:#?}", x.elapsed().as_millis());
                }
                PollState::Ready(_) => break,
            }
        }
    }
}
