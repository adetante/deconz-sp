use futures::sync::mpsc::{unbounded, UnboundedSender};
use futures::sync::oneshot::{channel, Sender};
use futures::{Future, Sink, Stream};
use log::*;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use tokio::codec::Framed;

use crate::call::Call;
use crate::protocol::constants::{NetworkStateCode, ParameterCode, StatusCode};
use crate::protocol::types::ParameterValue;
use crate::protocol::Codec;
use crate::protocol::{IncomingMessage, IncomingPayload, OutgoingMessage};
use crate::Error;

type SubscriptionId = (u8, u8); // (seq,command_code)

pub struct Client {
    sender: UnboundedSender<OutgoingMessage>,
    next_seq: RwLock<u8>,
    subscriptions: Arc<RwLock<BTreeMap<SubscriptionId, Sender<IncomingMessage>>>>,
}

impl Client {
    pub fn new(
        device_path: &'static str,
    ) -> Result<(Self, impl Stream<Item = IncomingPayload, Error = ()>), Error> {
        debug!("Connect to device {}...", device_path);
        let serial = tokio_serial::Serial::from_path(
            device_path,
            &mio_serial::SerialPortSettings {
                baud_rate: 38400,
                ..mio_serial::SerialPortSettings::default()
            },
        )?;
        debug!("Connected to device");
        let subscriptions: Arc<RwLock<BTreeMap<SubscriptionId, Sender<IncomingMessage>>>> =
            Arc::new(RwLock::new(BTreeMap::new()));
        let (sink, stream) = Framed::new(serial, Codec::new()).split();
        let (notif_tx, notif_rx) = unbounded();
        let (tx, rx) = unbounded();
        let forward_to_sink = rx.forward(sink.sink_map_err(|_| ())).map(|_| ());
        let subscriptions_ = subscriptions.clone();
        let process_stream = stream
            .for_each(move |message| {
                trace!("Received message: {:?}", message);
                let mut subscriptions = subscriptions_
                    .write()
                    .expect("Cannot obtain write-lock on subscriptions");
                if let Some(subscription) =
                    subscriptions.remove(&(message.seq, message.command.code()))
                {
                    trace!("Subscription exists!");
                    match subscription.send(message) {
                        Err(_) => {
                            error!("Cannot send message to receiver");
                            futures::future::err(Error::Internal("Cannot send message to receiver"))
                        }
                        Ok(_) => futures::future::ok(()),
                    }
                } else {
                    debug!("No subscription");
                    match notif_tx.clone().unbounded_send(message.payload) {
                        Err(_) => {
                            error!("Cannot send notification");
                            futures::future::err(Error::Internal("Cannot send notification"))
                        }
                        Ok(_) => futures::future::ok(()),
                    }
                }
            })
            .map_err(|err| {
                error!("Error occured while processing stream: {}", err);
                ()
            });
        tokio::spawn(forward_to_sink);
        tokio::spawn(process_stream);
        Ok((
            Self {
                sender: tx,
                next_seq: RwLock::new(0),
                subscriptions,
            },
            notif_rx,
        ))
    }

    fn send_request(
        &self,
        msg: OutgoingMessage,
    ) -> impl Future<Item = IncomingMessage, Error = Error> {
        let (sender, receiver) = channel();
        let mut subscriptions = self
            .subscriptions
            .write()
            .expect("Cannot get write-lock on subscription");
        (*subscriptions).insert((msg.seq, msg.command.code()), sender);
        self.sender
            .clone()
            .send(msg)
            .map_err(|error| {
                error!("Error while sending: {}", error);
                Error::Internal("Cannot send message")
            })
            .and_then(|_| Call::new(receiver))
    }

    pub fn read_parameter(
        &self,
        parameter: ParameterCode,
    ) -> impl Future<Item = ParameterValue, Error = Error> {
        let mut next_seq = self
            .next_seq
            .write()
            .expect("Cannot obtain write-lock on next_seq");
        let seq = *next_seq;
        *next_seq = next_seq.wrapping_add(1);
        self.send_request(OutgoingMessage::new_read_parameter(seq, parameter))
            .and_then(|response| match response.status {
                StatusCode::Success => match response.payload {
                    IncomingPayload::ReadParameter { value, .. } => futures::future::ok(value),
                    payload => futures::future::err(Error::UnexpectedResponsePayload(
                        "ReadParameter",
                        payload,
                    )),
                },
                status => futures::future::err(Error::NonSuccessResponse(status)),
            })
    }

    pub fn write_parameter(
        &self,
        parameter: ParameterCode,
        value: ParameterValue,
    ) -> impl Future<Item = (), Error = Error> {
        let mut next_seq = self
            .next_seq
            .write()
            .expect("Cannot obtain write-lock on next_seq");
        let seq = *next_seq;
        *next_seq = next_seq.wrapping_add(1);
        self.send_request(OutgoingMessage::new_write_parameter(seq, parameter, value))
            .and_then(|response| match response.status {
                StatusCode::Success => match response.payload {
                    IncomingPayload::WriteParameter { .. } => futures::future::ok(()),
                    payload => futures::future::err(Error::UnexpectedResponsePayload(
                        "WriteParameter",
                        payload,
                    )),
                },
                status => futures::future::err(Error::NonSuccessResponse(status)),
            })
    }

    pub fn device_state(&self) -> impl Future<Item = NetworkStateCode, Error = Error> {
        let mut next_seq = self
            .next_seq
            .write()
            .expect("Cannot obtain write-lock on next_seq");
        let seq = *next_seq;
        *next_seq = next_seq.wrapping_add(1);
        self.send_request(OutgoingMessage::new_device_state(seq))
            .and_then(|response| match response.status {
                StatusCode::Success => match response.payload {
                    IncomingPayload::DeviceState { state, .. } => futures::future::ok(state),
                    payload => futures::future::err(Error::UnexpectedResponsePayload(
                        "DeviceState",
                        payload,
                    )),
                },
                status => futures::future::err(Error::NonSuccessResponse(status)),
            })
    }

    pub fn change_network_state(
        &self,
        state: NetworkStateCode,
    ) -> impl Future<Item = (), Error = Error> {
        let mut next_seq = self
            .next_seq
            .write()
            .expect("Cannot obtain write-lock on next_seq");
        let seq = *next_seq;
        *next_seq = next_seq.wrapping_add(1);
        self.send_request(OutgoingMessage::new_change_network_state(seq, state))
            .and_then(|response| match response.status {
                StatusCode::Success => match response.payload {
                    IncomingPayload::ChangeNetworkState { .. } => futures::future::ok(()),
                    payload => futures::future::err(Error::UnexpectedResponsePayload(
                        "ChangeNetworkState",
                        payload,
                    )),
                },
                status => futures::future::err(Error::NonSuccessResponse(status)),
            })
    }

    pub fn aps_data_indication(&self) -> impl Future<Item = IncomingPayload, Error = Error> {
        let mut next_seq = self
            .next_seq
            .write()
            .expect("Cannot obtain write-lock on next_seq");
        let seq = *next_seq;
        *next_seq = next_seq.wrapping_add(1);
        self.send_request(OutgoingMessage::new_aps_data_indication(seq))
            .and_then(|response| match response.status {
                StatusCode::Success => match response.payload {
                    payload @ IncomingPayload::ApsDataIndication { .. } => {
                        futures::future::ok(payload)
                    }
                    payload => futures::future::err(Error::UnexpectedResponsePayload(
                        "ApsDataIndication",
                        payload,
                    )),
                },
                status => futures::future::err(Error::NonSuccessResponse(status)),
            })
    }
}
