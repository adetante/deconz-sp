use deconz_sp::{constants, types, IncomingPayload};
use futures::future::{Either, Future};
use futures::{self, stream::Stream};

fn main() {
    env_logger::init();
    tokio::run(futures::lazy(|| {
        let (client, notifications) = deconz_sp::Client::new("/dev/tty.usbserial-DM00ZSS9")
            .expect("Cannot initialize DeCONZ client");

        let client = std::sync::Arc::new(client);

        let set_security_mode = client.write_parameter(
            constants::ParameterCode::SecurityMode,
            types::ParameterValue::U8(1),
        );

        let handle_notifications = notifications.for_each(move |notif| match notif {
            // For each APSDE-DATA.indication, send a Read Received Data request
            IncomingPayload::DeviceState {
                apsde_data_indication,
                ..
            } if apsde_data_indication => Either::A(
                client
                    .clone()
                    .aps_data_indication()
                    .and_then(|data| {
                        println!("Data received: {:?}", data);
                        futures::future::ok(())
                    })
                    .map_err(|err| {
                        println!("Error occured when reading data: {}", err);
                        ()
                    }),
            ),
            // Unhandled notification
            _ => Either::B(futures::future::ok(())),
        });

        set_security_mode.then(|result| {
            match result {
                Err(error) => println!("Cannot change security mode: {}", error),
                Ok(_) => println!("SecurityMode changed"),
            };
            handle_notifications
        })
    }));
}
