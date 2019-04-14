use byteorder::{ByteOrder, LittleEndian};

use super::*;
use crate::protocol::constants::*;
use crate::protocol::types::*;

#[test]
fn encode_valid_read_parameter() {
    let request = OutgoingMessage::new_read_parameter(10, ParameterCode::ChannelMask);
    let mut output = [0; 32];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            println!("Frame len={} data={:x?}", len, &output);
            assert_eq!(len, 8, "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::ReadParameter.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]);
            assert_eq!(frame_len, 8, "Invalid fame len");
            let payload_len = LittleEndian::read_u16(&output[5..7]);
            assert_eq!(payload_len, 1, "Invalid payload len");
            assert_eq!(
                output[7],
                ParameterCode::ChannelMask.code(),
                "Invalid parameter"
            );
        }
    }
}

#[test]
fn encode_valid_write_parameter() {
    let request = OutgoingMessage::new_write_parameter(
        10,
        ParameterCode::NwkAddress,
        ParameterValue::U16(333),
    );
    let mut output = [0; 32];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            println!("Frame len={} data={:x?}", len, &output);
            assert_eq!(len, 10, "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::WriteParameter.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]);
            assert_eq!(frame_len, 10, "Invalid fame len");
            let payload_len = LittleEndian::read_u16(&output[5..7]);
            assert_eq!(payload_len, 3, "Invalid payload len");
            assert_eq!(
                output[7],
                ParameterCode::NwkAddress.code(),
                "Invalid parameter"
            );
            let parameter_value = LittleEndian::read_u16(&output[8..]);
            assert_eq!(parameter_value, 333, "Invalid parameter value");
        }
    }
}

#[test]
fn encode_valid_network_state() {
    let request = OutgoingMessage::new_device_state(10);
    let mut output = [0; 32];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            println!("Frame len={} data={:x?}", len, &output);
            assert_eq!(len, 8, "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::DeviceState.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]);
            assert_eq!(frame_len, 8, "Invalid fame len");
            assert_eq!(output[5], 0);
            assert_eq!(output[6], 0);
            assert_eq!(output[7], 0);
        }
    }
}

#[test]
fn encode_valid_change_network_state() {
    let request = OutgoingMessage::new_change_network_state(10, NetworkStateCode::Connected);
    let mut output = [0; 32];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            println!("Frame len={} data={:x?}", len, &output);
            assert_eq!(len, 6, "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::ChangeNetworkState.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]);
            assert_eq!(frame_len, 6, "Invalid fame len");
            assert_eq!(output[5], NetworkStateCode::Connected.code());
        }
    }
}

#[test]
fn encode_valid_aps_data_indication() {
    let request = OutgoingMessage::new_aps_data_indication(10);
    let mut output = [0; 32];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            println!("Frame len={} data={:x?}", len, &output);
            assert_eq!(len, 7, "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::ApsDataIndication.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]);
            assert_eq!(frame_len, 7, "Invalid fame len");
            let payload_len = LittleEndian::read_u16(&output[5..7]);
            assert_eq!(payload_len, 0, "Invalid payload len");
        }
    }
}

#[test]
fn encode_valid_aps_data_request_with_nkw() {
    let asdu: Vec<u8> = (0..255).collect();
    let request = OutgoingMessage::new_aps_data_request(
        10,
        100,
        Address::NWK(1, 2),
        10,
        11,
        12,
        13,
        asdu.clone(),
    );
    let mut output = [0; 300];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            let content: Vec<String> = output.iter().map(|v| format!("{:x}", v)).collect();
            println!("Frame len={} data=[{}]", len, content.join(","));
            assert_eq!(len, 22 + asdu.len(), "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::ApsDataRequest.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]) as usize;
            assert_eq!(frame_len, 22 + asdu.len(), "Invalid fame len");
            let payload_len = LittleEndian::read_u16(&output[5..7]) as usize;
            assert_eq!(payload_len, 15 + asdu.len(), "Invalid payload len");
            assert_eq!(output[7], 100, "Invalid request_id");
            assert_eq!(output[8], 0, "Invalid flag");
            assert_eq!(
                output[9],
                DestinationMode::NWK.code(),
                "Invalid destination mode"
            );
            let destination = LittleEndian::read_u16(&output[10..12]);
            assert_eq!(destination, 1, "Invalid destination");
            assert_eq!(output[12], 2, "Invalid endpoint");
            let profile_id = LittleEndian::read_u16(&output[13..15]);
            assert_eq!(profile_id, 10, "Invalid profile_id");
            let cluster_id = LittleEndian::read_u16(&output[15..17]);
            assert_eq!(cluster_id, 11, "Invalid cluster_id");
            assert_eq!(output[17], 12, "Invalid source_endpoint");
            let asdu_len = LittleEndian::read_u16(&output[18..20]) as usize;
            assert_eq!(asdu_len, asdu.len(), "Invalid asdu len");
            assert_eq!(&output[20..20 + asdu.len()], &asdu[0..], "Invalid asdu");
            assert_eq!(output[20 + asdu.len()], 0x4, "Invalid tx option");
            assert_eq!(output[20 + asdu.len() + 1], 13, "Invalid radius");
        }
    }
}

#[test]
fn encode_valid_aps_data_request_with_group() {
    let asdu: Vec<u8> = (0..255).collect();
    let request = OutgoingMessage::new_aps_data_request(
        10,
        100,
        Address::Group(1),
        10,
        11,
        12,
        13,
        asdu.clone(),
    );
    let mut output = [0; 300];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            let content: Vec<String> = output.iter().map(|v| format!("{:x}", v)).collect();
            println!("Frame len={} data=[{}]", len, content.join(","));
            assert_eq!(len, 21 + asdu.len(), "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::ApsDataRequest.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]) as usize;
            assert_eq!(frame_len, 21 + asdu.len(), "Invalid fame len");
            let payload_len = LittleEndian::read_u16(&output[5..7]) as usize;
            assert_eq!(payload_len, 14 + asdu.len(), "Invalid payload len");
            assert_eq!(output[7], 100, "Invalid request_id");
            assert_eq!(output[8], 0, "Invalid flag");
            assert_eq!(
                output[9],
                DestinationMode::Group.code(),
                "Invalid destination mode"
            );
            let destination = LittleEndian::read_u16(&output[10..12]);
            assert_eq!(destination, 1, "Invalid destination");
            let profile_id = LittleEndian::read_u16(&output[12..14]);
            assert_eq!(profile_id, 10, "Invalid profile_id");
            let cluster_id = LittleEndian::read_u16(&output[14..16]);
            assert_eq!(cluster_id, 11, "Invalid cluster_id");
            assert_eq!(output[16], 12, "Invalid source_endpoint");
            let asdu_len = LittleEndian::read_u16(&output[17..19]) as usize;
            assert_eq!(asdu_len, asdu.len(), "Invalid asdu len");
            assert_eq!(&output[19..19 + asdu.len()], &asdu[0..], "Invalid asdu");
            assert_eq!(output[19 + asdu.len()], 0x4, "Invalid tx option");
            assert_eq!(output[19 + asdu.len() + 1], 13, "Invalid radius");
        }
    }
}

#[test]
fn encode_valid_aps_data_request_with_ieee() {
    let asdu: Vec<u8> = (0..255).collect();
    let request = OutgoingMessage::new_aps_data_request(
        10,
        100,
        Address::IEEE(1, 2),
        10,
        11,
        12,
        13,
        asdu.clone(),
    );
    let mut output = [0; 300];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            let content: Vec<String> = output.iter().map(|v| format!("{:x}", v)).collect();
            println!("Frame len={} data=[{}]", len, content.join(","));
            assert_eq!(len, 28 + asdu.len(), "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::ApsDataRequest.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]) as usize;
            assert_eq!(frame_len, 28 + asdu.len(), "Invalid fame len");
            let payload_len = LittleEndian::read_u16(&output[5..7]) as usize;
            assert_eq!(payload_len, 21 + asdu.len(), "Invalid payload len");
            assert_eq!(output[7], 100, "Invalid request_id");
            assert_eq!(output[8], 0, "Invalid flag");
            assert_eq!(
                output[9],
                DestinationMode::IEEE.code(),
                "Invalid destination mode"
            );
            let destination = LittleEndian::read_u64(&output[10..18]);
            assert_eq!(destination, 1, "Invalid destination");
            assert_eq!(output[18], 2, "Invalid endpoint");
            let profile_id = LittleEndian::read_u16(&output[19..21]);
            assert_eq!(profile_id, 10, "Invalid profile_id");
            let cluster_id = LittleEndian::read_u16(&output[21..23]);
            assert_eq!(cluster_id, 11, "Invalid cluster_id");
            assert_eq!(output[23], 12, "Invalid source_endpoint");
            let asdu_len = LittleEndian::read_u16(&output[24..26]) as usize;
            assert_eq!(asdu_len, asdu.len(), "Invalid asdu len");
            assert_eq!(&output[26..26 + asdu.len()], &asdu[0..], "Invalid asdu");
            assert_eq!(output[26 + asdu.len()], 0x4, "Invalid tx option");
            assert_eq!(output[26 + asdu.len() + 1], 13, "Invalid radius");
        }
    }
}

#[test]
fn encode_valid_aps_data_confirm() {
    let request = OutgoingMessage::new_aps_data_confirm(10);
    let mut output = [0; 32];
    match request.write(&mut output) {
        Err(err) => panic!("Cannot write request: {}", err),
        Ok(len) => {
            println!("Frame len={} data={:x?}", len, &output);
            assert_eq!(len, 7, "Invalid frame len");
            assert_eq!(
                output[0],
                CommandCode::ApsDataConfirm.code(),
                "Invalid command"
            );
            assert_eq!(output[1], 10, "Invalid sequence");
            assert_eq!(output[2], 0);
            let frame_len = LittleEndian::read_u16(&output[3..5]);
            assert_eq!(frame_len, 7, "Invalid fame len");
            let payload_len = LittleEndian::read_u16(&output[5..7]);
            assert_eq!(payload_len, 0, "Invalid payload len");
        }
    }
}
