use log::*;
use futures::{ Future, Poll, Async, try_ready };
use futures::sync::oneshot::Receiver;
use crate::protocol::IncomingMessage;
use crate::Error;

pub struct Call {
  receiver: Receiver<IncomingMessage>
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