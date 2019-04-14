use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use log::*;
use serial_line_ip::{Decoder as SLIPDecoder, Encoder as SLIPEncoder};
use tokio::codec::{Decoder, Encoder};

use crate::protocol::{IncomingMessage, OutgoingMessage};
use crate::Error;

pub struct Codec {}

impl Codec {
    pub fn new() -> Self {
        Codec {}
    }
}

impl Decoder for Codec {
    type Item = IncomingMessage;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<IncomingMessage>, Error> {
        // TODO: validate CRC
        if buf.len() < 1 {
            return Ok(None);
        }
        trace!("Decode incoming frame: {:x?}", &buf as &[u8]);
        let mut decoder = SLIPDecoder::new();
        let mut output = [0; 300]; // TODO: 32 is suffisent?
        let (readed, _, is_end) = decoder.decode(buf, &mut output)?;
        trace!("SLIP-decoded frame: {:x?}", &output[0..readed]);
        if !is_end {
            trace!("Frame is not complete");
            return Ok(None);
        }
        buf.split_to(readed);
        match IncomingMessage::read(&output[0..readed]) {
            Ok(response) => {
                debug!("Decoded incoming frame: {:?}", response);
                Ok(Some(response))
            }
            Err(err) => {
                warn!("Receive invalid frame: {:?}", err);
                Ok(None)
            }
        }
    }
}

impl Encoder for Codec {
    type Item = OutgoingMessage;
    type Error = Error;

    fn encode(&mut self, msg: OutgoingMessage, buf: &mut BytesMut) -> Result<(), Error> {
        let mut data = [0; 300];
        let len = msg.write(&mut data)?;
        let crc = compute_crc(&data[0..len]);
        trace!(
            "Outgoing frame: {:x?} crc: {:x?}",
            &data[0..len] as &[u8],
            &crc
        );
        let mut encoder = SLIPEncoder::new();
        let mut output = [0; 600];
        let mut result = encoder.encode(&data[0..len], &mut output)?;
        result += encoder.encode(&crc, &mut output[result.1..])?;
        result += encoder.finish(&mut output[result.1..])?;
        trace!("SLIP encoded outgoing frame: {:x?}", &output[0..result.1]);
        buf.reserve(result.1);
        buf.put(&output[0..result.1]);
        debug!("Encoded outgoing frame: {:?}", msg);
        Ok(())
    }
}

fn compute_crc(data: &[u8]) -> [u8; 2] {
    let crc = data.iter().fold(0 as u16, |acc, value| acc + *value as u16);
    let crc = !(crc as u16) + 1;
    let mut buf = [0; 2];
    LittleEndian::write_u16(&mut buf, crc);
    buf
}
