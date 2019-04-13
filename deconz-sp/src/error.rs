use std::convert::From;
use failure::Fail;

use crate::protocol::constants::StatusCode;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "SLIP encoding error: {:?}", _0)]
    SLIP(serial_line_ip::Error),
    #[fail(display = "IO error")]
    IO(#[fail(cause)] std::io::Error),
    #[fail(display = "Encoding error: {}", _0)]
    Encoding(&'static str),
    #[fail(display = "Decoding error: {}", _0)]
    Decoding(&'static str),
    #[fail(display = "Generic error: {}", _0)]
    Generic(&'static str),
    #[fail(display = "Device returns non success code: {:?}", _0)]
    NonSuccessResponse(StatusCode)
}

impl From<std::io::Error> for Error {
  fn from(io: std::io::Error) -> Self {
    Error::IO(io)
  }
}

impl From<serial_line_ip::Error> for Error {
  fn from(slip: serial_line_ip::Error) -> Self {
    Error::SLIP(slip)
  }
}