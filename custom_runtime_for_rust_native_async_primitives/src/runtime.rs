pub use executor::{Executor, spawn};
pub use reactor::reactor;
mod executor;
mod reactor;
pub fn init() -> Executor {
    reactor::start();
    Executor::new()
}
