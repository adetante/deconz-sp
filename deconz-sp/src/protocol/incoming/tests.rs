use super::*;
use crate::protocol::constants::*;
use crate::protocol::types::*;

#[test]
fn decode_invalid_frame() {
    // Too short
    assert!(IncomingMessage::read(&[0x0, 0x1]).is_err());
    // Invalid frame len
    assert!(IncomingMessage::read(&[0x07, 0x1, 0x1, 0xff, 0x0, 0x0]).is_err());
    // Invalid status code
    assert!(IncomingMessage::read(&[0x07, 0x1, 0xff, 0x5, 0x0]).is_err());
    // Invalid command code
    assert!(IncomingMessage::read(&[0xff, 0x1, 0x1, 0x5, 0x0]).is_err());
}

#[test]
fn decode_valid_read_parameter() {
    let frame = [
        0xa, 0xa, 0x0, 0x10, 0x0, 0x9, 0x0, 0x1, 0xf, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ];
    let response = IncomingMessage::read(&frame);
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(
        response.command.code(),
        CommandCode::ReadParameter.code(),
        "Invalid command in response"
    );
    assert_eq!(response.seq, 10, "Invalid seq in response");
    assert_eq!(
        response.status.code(),
        StatusCode::Success.code(),
        "Invalid status in response"
    );
    match response.payload {
        IncomingPayload::ReadParameter { parameter, value } => {
            assert_eq!(
                parameter.code(),
                ParameterCode::MacAddress.code(),
                "Invalid parameter"
            );
            assert_eq!(value.length(), 8, "Invalid parameter len");
            assert_eq!(value.u64(), 15, "Invalid parameter value");
        }
        _ => panic!("Invalid response payload"),
    };
}

#[test]
fn decode_valid_write_parameter() {
    let frame = [0xb, 0xa, 0x0, 0x8, 0x0, 0x1, 0x0, 0x9];
    let response = IncomingMessage::read(&frame);
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(
        response.command.code(),
        CommandCode::WriteParameter.code(),
        "Invalid command in response"
    );
    assert_eq!(response.seq, 10, "Invalid seq in response");
    assert_eq!(
        response.status.code(),
        StatusCode::Success.code(),
        "Invalid status in response"
    );
    match response.payload {
        IncomingPayload::WriteParameter { parameter } => {
            assert_eq!(
                parameter.code(),
                ParameterCode::ApsDesignedCoordinator.code(),
                "Invalid parameter"
            );
        }
        _ => panic!("Invalid response payload"),
    };
}

#[test]
fn decode_valid_device_state() {
    let frame = [0x7, 0xa, 0x0, 0x8, 0x0, 0x2 & 0x3, 0x0, 0x0];
    let response = IncomingMessage::read(&frame);
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(
        response.command.code(),
        CommandCode::DeviceState.code(),
        "Invalid command in response"
    );
    assert_eq!(response.seq, 10, "Invalid seq in response");
    assert_eq!(
        response.status.code(),
        StatusCode::Success.code(),
        "Invalid status in response"
    );
    match response.payload {
        IncomingPayload::DeviceState { state, .. } => {
            assert_eq!(
                state.code(),
                NetworkStateCode::Connected.code(),
                "Invalid parameter"
            );
        }
        _ => panic!("Invalid response payload"),
    };
}

#[test]
fn decode_valid_change_network_state() {
    let frame = [0x8, 0xa, 0x0, 0x6, 0x0, 0x2 & 0x3, 0x0, 0x0];
    let response = IncomingMessage::read(&frame);
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(
        response.command.code(),
        CommandCode::ChangeNetworkState.code(),
        "Invalid command in response"
    );
    assert_eq!(response.seq, 10, "Invalid seq in response");
    assert_eq!(
        response.status.code(),
        StatusCode::Success.code(),
        "Invalid status in response"
    );
    match response.payload {
        IncomingPayload::ChangeNetworkState { state } => {
            assert_eq!(
                state.code(),
                NetworkStateCode::Connected.code(),
                "Invalid parameter"
            );
        }
        _ => panic!("Invalid response payload"),
    };
}

#[test]
fn decode_valid_aps_data_indication() {
    let mut payload_len = [0; 2];
    LittleEndian::write_u16(&mut payload_len, 26);
    let mut frame_len = [0; 2];
    LittleEndian::write_u16(&mut frame_len, 33);
    let frame = [
        0x17,
        0xa,
        0x0,
        frame_len[0],
        frame_len[1],
        payload_len[0],
        payload_len[1],
        0x0,
        0x1,
        0x1,
        0x0,
        0x0,
        0x2,
        0x2,
        0x0,
        0x4,
        0x1,
        0x0,
        0x2,
        0x0,
        0x3,
        0x0,
        0x1,
        0x2,
        0x3,
        0x0,
        0x0,
        0x5,
        0x0,
        0x0,
        0x0,
        0x0,
        0x6,
    ];
    let response = IncomingMessage::read(&frame);
    println!("{:?}", response);
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(
        response.command.code(),
        CommandCode::ApsDataIndication.code(),
        "Invalid command in response"
    );
    assert_eq!(response.seq, 10, "Invalid seq in response");
    assert_eq!(
        response.status.code(),
        StatusCode::Success.code(),
        "Invalid status in response"
    );
    match response.payload {
        IncomingPayload::ApsDataIndication {
            source,
            destination,
            profile_id,
            cluster_id,
            asdu,
            lqi,
            rssi,
        } => {
            match source {
                Address::NWK(addr, endpoint) => {
                    assert_eq!(addr, 2);
                    assert_eq!(endpoint, 4);
                }
                _ => panic!("Invalid mode for source address"),
            };
            match destination {
                Address::Group(addr) => {
                    assert_eq!(addr, 1);
                }
                _ => panic!("Invalid mode for destinations address"),
            };
            assert_eq!(profile_id, 1);
            assert_eq!(cluster_id, 2);
            assert_eq!(asdu, [0x1, 0x2, 0x3]);
            assert_eq!(lqi, 5);
            assert_eq!(rssi, 6);
        }
        _ => panic!("Invalid response payload"),
    };
}
