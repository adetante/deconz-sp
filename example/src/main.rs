use futures::{ self, future::Future };
use deconz_sp::constants;

fn main() {
    env_logger::init();
    tokio::run(futures::lazy(||{
        let client = deconz_sp::Client::new("/dev/tty.usbserial-DM00ZSS9").expect("Cannot initialize DeCONZ client");

        let get_security_mode = client.read_parameter(constants::ParameterCode::SecurityMode).then(|result| {
            match &result {
                Err(error) => println!("Cannot read parameter value: {}", error),
                Ok(value) => println!("SecurityMode = {:?}", value)
            };
            result
        });

        let get_device_state = client.device_state().then(|result| {
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
}
