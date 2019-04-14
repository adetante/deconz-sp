use crate::protocol::IncomingMessage;
use crate::Error;
use futures::sync::oneshot::Receiver;
use futures::{try_ready, Async, Future, Poll};
use log::*;

pub struct Call {
    receiver: Receiver<IncomingMessage>,
}

impl Call {
    pub fn new(receiver: Receiver<IncomingMessage>) -> Self {
        Call { receiver }
    }
}

impl Future for Call {
    type Item = IncomingMessage;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let msg = try_ready!(self.receiver.poll().map_err(|err| {
            error!("Channel canceled: {}", err);
            Error::Internal("Channel canceled")
        }));
        Ok(Async::Ready(msg))
    }
}
