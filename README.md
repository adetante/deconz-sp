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

`deconz_sp::Client` wraps the communication with the device.  
The Tokio runtime must be started to use the client.

```rust
tokio::run(futures::lazy(||{
        let client = deconz_sp::Client::new("/dev/tty.usbserial-DM00ZSS9")
        					.expect("Cannot initialize DeCONZ client");

        let get_security_mode = client.read_parameter(constants::ParameterCode::SecurityMode)
        	.then(|result| {
	            match &result {
	                Err(error) => println!("Cannot read parameter value: {}", error),
	                Ok(value) => println!("SecurityMode = {:?}", value)
	            };
	            result
   		     });

        let get_device_state = client.device_state()
        	.then(|result| {
	            match &result {
	                Err(error) => println!("Cannot read device state: {}", error),
	                Ok(value) => println!("DeviceState = {:?}", value)
	            };
	            result
        });;

        get_security_mode
            .and_then(|_| get_device_state)
            .map(|_|()).map_err(|_|())
    }));
```


Run the example: 

```
RUST_LOG=deconz_sp=TRACE cargo run
```

## Built With

* [tokio](https://tokio.rs/) asynchronous run-time 
* [tokio_serial](https://docs.rs/tokio-serial) and [mio_serial](https://docs.rs/mio-serial) for serial I/O