mod http;
mod runtime;
use crate::http::Http;
use isahc::prelude::*;
use tokio::runtime::Runtime;
fn main() {
    let mut executor = runtime::init();
    executor.block_on(async_main())
}

async fn async_main() {
    println!("Program starting");
    let url = "http://127.0.0.1:8080/600/HelloAsyncAwait1";
    let mut res = isahc::get_async(url).await.unwrap();
    let txt = res.text().await.unwrap();
    println!("{txt}");
    let url = "http://127.0.0.1:8080/400/HelloAsyncAwait2";
    let mut res = isahc::get_async(url).await.unwrap();
    let txt = res.text().await.unwrap();
    println!("{txt}");
}
