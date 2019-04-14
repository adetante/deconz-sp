# deconz-sp

A tokio-based Rust library that implements deCONZ Serial Protocol to interact with deCONZ devices.

Based on [Serial Protocol specifications](http://www.dresden-elektronik.de/rpi/deconz/deCONZ-Serial-Protocol-en.pdf), tested with the [ConBee USB gateway](https://www.dresden-elektronik.de/conbee).

**:warning: Work in progress, library is not finalized. Do not use in production. :warning:**


## Build

```
cargo build
cargo test
```

## Example

See `example/src/main.rs` for a working example.

`deconz_sp::Client` wraps the communication with the device.  
The Tokio runtime must be started to use the client.

```rust
tokio::run(futures::lazy(|| {
        let (client, notifications) = deconz_sp::Client::new("/dev/tty.usbserial-DM00ZSS9")
            .expect("Cannot initialize deCONZ client");
        // ...
}))
```

`deconz_sp::Client::new` returns a tuple `(Client, Stream<Item = IncomingPayload>)` where `Client` is used to send requests to device, and `Stream` is the stream of unsolicited received messages.

Run the example:
```
RUST_LOG=deconz_sp=TRACE cargo run
```

## Built With

* [tokio](https://tokio.rs/) asynchronous run-time 
* [tokio_serial](https://docs.rs/tokio-serial) and [mio_serial](https://docs.rs/mio-serial) for serial I/O