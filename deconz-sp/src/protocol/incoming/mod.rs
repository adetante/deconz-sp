use super::constants::{CommandCode, NetworkStateCode, ParameterCode, StatusCode};
use super::types::{Address, ParameterValue};
use crate::Error;
use byteorder::{ByteOrder, LittleEndian};
use log::*;

#[cfg(test)]
mod tests;

const FRAME_MIN_LEN: usize = 5;

#[derive(Debug)]
pub enum IncomingPayload {
    ReadParameter {
        parameter: ParameterCode,
        value: ParameterValue,
    },
    WriteParameter {
        parameter: ParameterCode,
    },
    DeviceState {
        state: NetworkStateCode,
        apsde_data_confirm: bool,
        apsde_data_indication: bool,
        configuration_changed: bool,
        apsde_data_request: bool,
    },
    ChangeNetworkState {
        state: NetworkStateCode,
    },
    ApsDataIndication {
        source: Address,
        destination: Address,
        profile_id: u16,
        cluster_id: u16,
        asdu: Vec<u8>,
        lqi: u8,
        rssi: i8,
    },
}

fn decode_device_state(state: u8) -> Option<(NetworkStateCode, bool, bool, bool, bool)> {
    let network_state = NetworkStateCode::from_code(state & 0x3)?;
    let apsde_data_confirm = (state & 0x4) != 0;
    let apsde_data_indication = (state & 0x8) != 0;
    let configuration_changed = (state & 0x10) != 0;
    let apsde_data_request = (state & 0x20) != 0;
    Some((
        network_state,
        apsde_data_confirm,
        apsde_data_indication,
        configuration_changed,
        apsde_data_request,
    ))
}

impl IncomingPayload {
    fn read(command: &CommandCode, input: &[u8]) -> Result<Self, Error> {
        match command {
            CommandCode::ReadParameter => {
                if input.len() <= 3 {
                    return Err(Error::Decoding("Too short payload for ReadParameter"));
                }
                let payload_len = LittleEndian::read_u16(&input[0..2]) as usize;
                if input.len() < payload_len {
                    return Err(Error::Decoding(
                        "Too short payload: incorrect payload length",
                    ));
                }
                let parameter_len = payload_len - 1;
                let parameter = ParameterCode::from_code(input[2]);
                let parameter_value =
                    LittleEndian::read_uint(&input[3..3 + parameter_len], parameter_len);
                match parameter {
                    None => Err(Error::Decoding("Unknown parameter id")),
                    Some(parameter) => Ok(IncomingPayload::ReadParameter {
                        parameter,
                        value: ParameterValue::from_value_and_len(parameter_value, parameter_len),
                    }),
                }
            }
            CommandCode::WriteParameter => {
                if input.len() < 3 {
                    return Err(Error::Decoding("Too short payload for WriteParameter"));
                }
                let payload_len = LittleEndian::read_u16(&input[0..2]) as usize;
                if payload_len < 1 {
                    return Err(Error::Decoding("Invalid payload length for WriteParameter"));
                }
                match ParameterCode::from_code(input[2]) {
                    None => Err(Error::Decoding("Unknown parameter id")),
                    Some(parameter) => Ok(IncomingPayload::WriteParameter { parameter }),
                }
            }
            CommandCode::DeviceState => {
                if input.len() < 1 {
                    return Err(Error::Decoding("Too short payload for DeviceState"));
                }
                match decode_device_state(input[0]) {
                    None => Err(Error::Decoding("Cannot decode device state")),
                    Some((
                        state,
                        apsde_data_confirm,
                        apsde_data_indication,
                        configuration_changed,
                        apsde_data_request,
                    )) => Ok(IncomingPayload::DeviceState {
                        state,
                        apsde_data_confirm,
                        apsde_data_indication,
                        configuration_changed,
                        apsde_data_request,
                    }),
                }
            }
            CommandCode::ChangeNetworkState => {
                if input.len() < 1 {
                    return Err(Error::Decoding("Too short payload for DeviceState"));
                }
                match NetworkStateCode::from_code(input[0]) {
                    None => Err(Error::Decoding("Unknown network state")),
                    Some(state) => Ok(IncomingPayload::ChangeNetworkState { state }),
                }
            }
            CommandCode::ApsDataIndication => {
                if input.len() < 2 {
                    return Err(Error::Decoding("Too short payload for ApsDataIndication"));
                }
                let payload_length = LittleEndian::read_u16(&input) as usize;
                if input.len() < (2 + payload_length) {
                    return Err(Error::Decoding(
                        "Too short payload for ApsDataIndication: invalid payload_length",
                    ));
                }
                let device_state = input[2];
                debug!("DeviceState: {}", device_state);
                let (destination, next_offset) = match input[3] {
                    0x1 => {
                        let address = LittleEndian::read_u16(&input[4..6]);
                        debug!("Endpoint ? : {}", input[6]);
                        (Address::Group(address), 7)
                    }
                    0x2 => {
                        let address = LittleEndian::read_u16(&input[4..6]);
                        let endpoint = input[6];
                        (Address::NWK(address, endpoint), 7)
                    }
                    0x3 => {
                        let address = LittleEndian::read_u64(&input[4..12]);
                        let endpoint = input[12];
                        (Address::IEEE(address, endpoint), 13)
                    }
                    _ => return Err(Error::Decoding("Unknown address mode for destination")),
                };
                let (source, next_offset) = match input[next_offset] {
                    0x1 => {
                        let address =
                            LittleEndian::read_u16(&input[next_offset + 1..next_offset + 3]);
                        debug!("Endpoint ? : {}", input[next_offset + 3]);
                        (Address::Group(address), next_offset + 4)
                    }
                    0x2 => {
                        let address =
                            LittleEndian::read_u16(&input[next_offset + 1..next_offset + 3]);
                        let endpoint = input[next_offset + 3];
                        (Address::NWK(address, endpoint), next_offset + 4)
                    }
                    0x3 => {
                        let address =
                            LittleEndian::read_u64(&input[next_offset + 1..next_offset + 9]);
                        let endpoint = input[next_offset + 9];
                        (Address::IEEE(address, endpoint), next_offset + 10)
                    }
                    _ => return Err(Error::Decoding("Unknown address mode for source")),
                };
                let profile_id = LittleEndian::read_u16(&input[next_offset..next_offset + 2]);
                let cluster_id = LittleEndian::read_u16(&input[next_offset + 2..next_offset + 4]);
                let asdu_len =
                    LittleEndian::read_u16(&input[next_offset + 4..next_offset + 6]) as usize;
                let asdu = Vec::from(&input[next_offset + 6..next_offset + 6 + asdu_len]);
                let next_offset = next_offset + 6 + asdu_len;
                let lqi = input[next_offset + 2];
                let rssi = input[next_offset + 7] as i8;
                Ok(IncomingPayload::ApsDataIndication {
                    source,
                    destination,
                    profile_id,
                    cluster_id,
                    asdu,
                    lqi,
                    rssi,
                })
            }
            CommandCode::DeviceStateChanged => match decode_device_state(input[0]) {
                None => Err(Error::Decoding("Cannot decode device state")),
                Some((
                    state,
                    apsde_data_confirm,
                    apsde_data_indication,
                    configuration_changed,
                    apsde_data_request,
                )) => Ok(IncomingPayload::DeviceState {
                    state,
                    apsde_data_confirm,
                    apsde_data_indication,
                    configuration_changed,
                    apsde_data_request,
                }),
            },
            _ => Err(Error::Decoding(
                "This command decoder is not yet implemented",
            )),
        }
    }
}

#[derive(Debug)]
pub struct IncomingMessage {
    pub command: CommandCode,
    pub seq: u8,
    pub status: StatusCode,
    pub payload: IncomingPayload,
}

impl IncomingMessage {
    pub fn read(input: &[u8]) -> Result<Self, Error> {
        if input.len() < FRAME_MIN_LEN {
            return Err(Error::Decoding("Frame is too short: cannot read header"));
        }
        let command = CommandCode::from_code(input[0]);
        let seq = input[1];
        let status = StatusCode::from_code(input[2]);
        let frame_len = LittleEndian::read_u16(&input[3..5]) as usize;
        if input.len() < frame_len {
            return Err(Error::Decoding(
                "Frame is too short: invalid Frame length value",
            ));
        }
        match (command, status) {
            (None, _) => Err(Error::Decoding("Invalid commande code")),
            (_, None) => Err(Error::Decoding("Invalid status code")),
            (Some(command), Some(status)) => {
                let payload = IncomingPayload::read(&command, &input[5..frame_len])?;
                Ok(IncomingMessage {
                    command,
                    seq,
                    status,
                    payload,
                })
            }
        }
    }
}
