use byteorder::{ByteOrder, LittleEndian};

use crate::protocol::constants::{CommandCode, NetworkStateCode, ParameterCode};
use crate::protocol::types::{Address, ParameterValue};
use crate::Error;

#[cfg(test)]
mod tests;

const FRAME_MIN_LEN: usize = 5;

#[derive(Debug)]
enum OutgoingPayload {
    Empty,
    ReadParameter {
        parameter: ParameterCode,
    },
    WriteParameter {
        parameter: ParameterCode,
        value: ParameterValue,
    },
    DeviceState,
    ChangeNetworkState {
        state: NetworkStateCode,
    },
    ApsDataRequest {
        request_id: u8,
        destination: Address,
        profile_id: u16,
        cluster_id: u16,
        source_endpoint: u8,
        asdu: Vec<u8>,
        radius: u8,
    },
}

impl OutgoingPayload {
    fn has_variable_length(&self) -> bool {
        match self {
            OutgoingPayload::Empty => true,
            OutgoingPayload::ReadParameter { .. } => true,
            OutgoingPayload::WriteParameter { .. } => true,
            OutgoingPayload::ApsDataRequest { .. } => true,
            _ => false,
        }
    }
    fn length(&self) -> usize {
        match self {
            OutgoingPayload::Empty => 0,
            OutgoingPayload::ReadParameter { .. } => 1,
            OutgoingPayload::WriteParameter { value, .. } => 1 + value.length(),
            OutgoingPayload::DeviceState => 3,
            OutgoingPayload::ChangeNetworkState { .. } => 1,
            OutgoingPayload::ApsDataRequest {
                destination, asdu, ..
            } => {
                let address_len = match destination {
                    Address::Group(_) => 2,
                    Address::NWK(_, _) => 3,
                    Address::IEEE(_, _) => 9,
                };
                12 + address_len + asdu.len()
            }
        }
    }
    fn write(&self, out: &mut [u8]) -> Result<(), Error> {
        match self {
            OutgoingPayload::Empty => Ok(()),
            OutgoingPayload::ReadParameter { parameter } => {
                out[0] = parameter.code();
                Ok(())
            }
            OutgoingPayload::WriteParameter { parameter, value } => {
                out[0] = parameter.code();
                LittleEndian::write_uint(&mut out[1..], value.u64(), value.length());
                Ok(())
            }
            OutgoingPayload::DeviceState => {
                out[0..3].clone_from_slice(&[0x0, 0x0, 0x0]);
                Ok(())
            }
            OutgoingPayload::ChangeNetworkState { state } => {
                out[0] = state.code();
                Ok(())
            }
            OutgoingPayload::ApsDataRequest {
                request_id,
                destination,
                profile_id,
                cluster_id,
                source_endpoint,
                asdu,
                radius,
            } => {
                out[0] = *request_id;
                out[1] = 0x0;
                out[2] = destination.mode().code();
                let mut next_offset = match destination {
                    Address::Group(addr) => {
                        LittleEndian::write_u16(&mut out[3..5], *addr);
                        5
                    }
                    Address::NWK(addr, endpoint) => {
                        LittleEndian::write_u16(&mut out[3..5], *addr);
                        out[5] = *endpoint;
                        6
                    }
                    Address::IEEE(addr, endpoint) => {
                        LittleEndian::write_u64(&mut out[3..11], *addr);
                        out[11] = *endpoint;
                        12
                    }
                };
                LittleEndian::write_u16(&mut out[next_offset..next_offset + 2], *profile_id);
                next_offset += 2;
                LittleEndian::write_u16(&mut out[next_offset..next_offset + 2], *cluster_id);
                next_offset += 2;
                out[next_offset] = *source_endpoint;
                next_offset += 1;
                LittleEndian::write_u16(&mut out[next_offset..next_offset + 2], asdu.len() as u16);
                next_offset += 2;
                out[next_offset..next_offset + asdu.len()].clone_from_slice(&asdu);
                next_offset += asdu.len();
                out[next_offset] = 0x4;
                next_offset += 1;
                out[next_offset] = *radius;
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct OutgoingMessage {
    pub command: CommandCode,
    pub seq: u8,
    payload: OutgoingPayload,
}

impl OutgoingMessage {
    pub fn new_read_parameter(seq: u8, parameter: ParameterCode) -> Self {
        OutgoingMessage {
            command: CommandCode::ReadParameter,
            seq,
            payload: OutgoingPayload::ReadParameter { parameter },
        }
    }

    pub fn new_write_parameter(seq: u8, parameter: ParameterCode, value: ParameterValue) -> Self {
        OutgoingMessage {
            command: CommandCode::WriteParameter,
            seq,
            payload: OutgoingPayload::WriteParameter { parameter, value },
        }
    }

    pub fn new_device_state(seq: u8) -> Self {
        OutgoingMessage {
            command: CommandCode::DeviceState,
            seq,
            payload: OutgoingPayload::DeviceState,
        }
    }

    pub fn new_change_network_state(seq: u8, state: NetworkStateCode) -> Self {
        OutgoingMessage {
            command: CommandCode::ChangeNetworkState,
            seq,
            payload: OutgoingPayload::ChangeNetworkState { state },
        }
    }

    pub fn new_aps_data_indication(seq: u8) -> Self {
        OutgoingMessage {
            command: CommandCode::ApsDataIndication,
            seq,
            payload: OutgoingPayload::Empty,
        }
    }

    pub fn new_aps_data_request(
        seq: u8,
        request_id: u8,
        destination: Address,
        profile_id: u16,
        cluster_id: u16,
        source_endpoint: u8,
        radius: u8,
        asdu: Vec<u8>,
    ) -> Self {
        OutgoingMessage {
            command: CommandCode::ApsDataRequest,
            seq,
            payload: OutgoingPayload::ApsDataRequest {
                request_id,
                destination,
                profile_id,
                cluster_id,
                source_endpoint,
                asdu,
                radius,
            },
        }
    }

    pub fn new_aps_data_confirm(seq: u8) -> Self {
        OutgoingMessage {
            command: CommandCode::ApsDataConfirm,
            seq,
            payload: OutgoingPayload::Empty,
        }
    }

    pub fn write(&self, out: &mut [u8]) -> Result<usize, Error> {
        let frame_len = if self.payload.has_variable_length() {
            FRAME_MIN_LEN + 2 + self.payload.length()
        } else {
            FRAME_MIN_LEN + self.payload.length()
        };
        if out.len() < frame_len {
            return Err(Error::Encoding("Not enougth space for encoding this frame"));
        }
        out[0] = self.command.code();
        out[1] = self.seq;
        out[2] = 0x0;
        LittleEndian::write_u16(&mut out[3..5], frame_len as u16);
        let next_offset = if self.payload.has_variable_length() {
            LittleEndian::write_u16(&mut out[5..7], self.payload.length() as u16);
            7
        } else {
            5
        };
        self.payload.write(&mut out[next_offset..])?;
        Ok(frame_len)
    }
}
