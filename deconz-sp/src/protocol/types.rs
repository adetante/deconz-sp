use crate::protocol::constants::DestinationMode;

#[derive(Debug)]
pub enum Address {
  Group(u16),
  NWK(u16, u8),
  IEEE(u64, u8)
}

impl Address {
  pub fn mode(&self) -> DestinationMode {
    match self {
      Address::Group(_) => DestinationMode::Group,
      Address::NWK(_,_) => DestinationMode::NWK,
      Address::IEEE(_,_) => DestinationMode::IEEE
    }
  }
}

#[derive(Debug)]
pub enum ParameterValue {
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64)
}

impl ParameterValue {
  pub fn length(&self) -> usize {
    match self {
      ParameterValue::U8(_) => 1,
      ParameterValue::U16(_) => 2,
      ParameterValue::U32(_) => 4,
      ParameterValue::U64(_) => 8
    }
  }

  pub fn u64(&self) -> u64 {
    match self {
      ParameterValue::U8(value) => *value as u64,
      ParameterValue::U16(value) => *value as u64,
      ParameterValue::U32(value) => *value as u64,
      ParameterValue::U64(value) => *value
    }
  }
}

impl std::convert::From<u8> for ParameterValue {
  fn from(value: u8) -> Self {
    ParameterValue::U8(value)
  }
}

impl std::convert::From<u16> for ParameterValue {
  fn from(value: u16) -> Self {
    ParameterValue::U16(value)
  }
}

impl std::convert::From<u32> for ParameterValue {
  fn from(value: u32) -> Self {
    ParameterValue::U32(value)
  }
}

impl std::convert::From<u64> for ParameterValue {
  fn from(value: u64) -> Self {
    ParameterValue::U64(value)
  }
}