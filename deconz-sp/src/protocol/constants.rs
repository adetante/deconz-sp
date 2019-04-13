#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum CommandCode {
  DeviceState,
  ChangeNetworkState,
  ReadParameter,
  WriteParameter,
  DeviceStateChanged,
  ApsDataRequest,
  ApsDataConfirm,
  ApsDataIndication,
}

impl CommandCode {
  pub fn code(&self) -> u8 {
    match self {
      CommandCode::DeviceState => 0x07,
		  CommandCode::ChangeNetworkState => 0x08,
		  CommandCode::ReadParameter => 0x0a,
		  CommandCode::WriteParameter => 0x0b,
		  CommandCode::DeviceStateChanged => 0x0e,
		  CommandCode::ApsDataRequest => 0x12,
		  CommandCode::ApsDataConfirm => 0x04,
		  CommandCode::ApsDataIndication => 0x17
    }
  }
  pub fn from_code(code: u8) -> Option<Self> {
    match code {
      0x07 => Some(CommandCode::DeviceState),
      0x08 => Some(CommandCode::ChangeNetworkState),
		  0x0a => Some(CommandCode::ReadParameter),
		  0x0b => Some(CommandCode::WriteParameter),
		  0x0e => Some(CommandCode::DeviceStateChanged),
		  0x12 => Some(CommandCode::ApsDataRequest),
		  0x04 => Some(CommandCode::ApsDataConfirm),
		  0x17 => Some(CommandCode::ApsDataIndication),
      _ => None
    }
  }
}

#[derive(Debug)]
pub enum ParameterCode {
  MacAddress,
	NwkPanId,
	NwkAddress,
	NwkExtendedPanId,
	ApsDesignedCoordinator,
	ChannelMask,
	ApsExtendedPanId,
	TrustCenterAddress,
	SecurityMode,
	NetworkKey,
	CurrentChannel,
  ProtocolVersion,
	NwkUpdateId
}

impl ParameterCode {
  pub fn code(&self) -> u8 {
    match self {
      ParameterCode::MacAddress => 0x01,
      ParameterCode::NwkPanId => 0x05,
      ParameterCode::NwkAddress => 0x07,
      ParameterCode::NwkExtendedPanId => 0x08,
      ParameterCode::ApsDesignedCoordinator => 0x09,
      ParameterCode::ChannelMask => 0x0a,
      ParameterCode::ApsExtendedPanId => 0x0b,
      ParameterCode::TrustCenterAddress => 0x0e,
      ParameterCode::SecurityMode => 0x10,
      ParameterCode::NetworkKey => 0x18,
      ParameterCode::CurrentChannel => 0x1c,
      ParameterCode::ProtocolVersion => 0x22,
      ParameterCode::NwkUpdateId => 0x24
    }
  }
  pub fn from_code(code: u8) -> Option<Self> {
    match code {
       0x01 => Some(ParameterCode::MacAddress),
       0x05 => Some(ParameterCode::NwkPanId),
       0x07 => Some(ParameterCode::NwkAddress),
       0x08 => Some(ParameterCode::NwkExtendedPanId),
       0x09 => Some(ParameterCode::ApsDesignedCoordinator),
       0x0a => Some(ParameterCode::ChannelMask),
       0x0b => Some(ParameterCode::ApsExtendedPanId),
       0x0e => Some(ParameterCode::TrustCenterAddress),
       0x10 => Some(ParameterCode::SecurityMode),
       0x18 => Some(ParameterCode::NetworkKey),
       0x1c => Some(ParameterCode::CurrentChannel),
       0x22 => Some(ParameterCode::ProtocolVersion),
       0x24 => Some(ParameterCode::NwkUpdateId),
       _ => None
    }
  }
  pub fn len(&self) -> u8 {
    match self {
      ParameterCode::MacAddress => 8,
      ParameterCode::NwkPanId => 2,
      ParameterCode::NwkAddress => 2,
      ParameterCode::NwkExtendedPanId => 8,
      ParameterCode::ApsDesignedCoordinator => 1,
      ParameterCode::ChannelMask => 4,
      ParameterCode::ApsExtendedPanId => 8,
      ParameterCode::TrustCenterAddress => 8,
      ParameterCode::SecurityMode => 1,
      ParameterCode::NetworkKey => 1,
      ParameterCode::CurrentChannel => 1,
      ParameterCode::ProtocolVersion => 2,
      ParameterCode::NwkUpdateId => 1
    }
  }
}

#[derive(Debug)]
pub enum StatusCode {
  Success,
  Failure,
  Busy,
  Timeout,
  Unsupported,
  Error,
  NoNetwork,
  InvalidValue,
  ModuleResponseTimeout,
  ModuleSendDataTimeout
}

impl StatusCode {
  pub fn code(&self) -> u8 {
    match self {
      StatusCode::Success => 0,
      StatusCode::Failure => 1,
      StatusCode::Busy => 2,
      StatusCode::Timeout => 3,
      StatusCode::Unsupported => 4,
      StatusCode::Error => 5,
      StatusCode::NoNetwork => 6,
      StatusCode::InvalidValue => 7,
      StatusCode::ModuleResponseTimeout => 8,
      StatusCode::ModuleSendDataTimeout => 9
    }
  }
  pub fn from_code(code: u8) -> Option<Self> {
    match code {
      0 => Some(StatusCode::Success),
      1 => Some(StatusCode::Failure),
      2 => Some(StatusCode::Busy),
      3 => Some(StatusCode::Timeout),
      4 => Some(StatusCode::Unsupported),
      5 => Some(StatusCode::Error),
      6 => Some(StatusCode::NoNetwork),
      7 => Some(StatusCode::InvalidValue),
      8 => Some(StatusCode::ModuleResponseTimeout),
      9 => Some(StatusCode::ModuleSendDataTimeout),
      _ => None
    }
  }
}

#[derive(Debug)]
pub enum NetworkStateCode {
  Offline,
  Joining,
  Connected,
  Leaving,
}

impl NetworkStateCode {
  pub fn code(&self) -> u8 {
    match self {
      NetworkStateCode::Offline => 0,
      NetworkStateCode::Joining => 1,
      NetworkStateCode::Connected => 2,
      NetworkStateCode::Leaving => 3
    }
  }
  pub fn from_code(code: u8) -> Option<Self> {
    match code {
      0 => Some(NetworkStateCode::Offline),
      1 => Some(NetworkStateCode::Joining),
      2 => Some(NetworkStateCode::Connected),
      3 => Some(NetworkStateCode::Leaving),
      _ => None
    }
  }
}

#[derive(Debug)]
pub enum DestinationMode {
  Group,
  NWK,
  IEEE
}

impl DestinationMode {
  pub fn code(&self) -> u8 {
    match self {
      DestinationMode::Group => 0x1,
      DestinationMode::NWK => 0x2,
      DestinationMode::IEEE => 0x3
    }
  }
}