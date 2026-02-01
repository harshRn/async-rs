changes compared to the source:
1. instead of tracking what stream has had it's event handled already in a-epoll, I proceed with the de-registration of the stream as a tracked source : https://github.com/PacktPublishing/Asynchronous-Programming-in-Rust/issues/4
2. https://github.com/PacktPublishing/Asynchronous-Programming-in-Rust/issues/45
