use std::io::Cursor;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use enumflags2::BitFlags;

use super::cache_change::ChangeKind;
use crate::{
  messages::submessages::submessage_elements::RepresentationIdentifier,
  serialization::{
    CDRDeserializerAdapter, DeserializerAdapter,
    cdr_serializer::{to_bytes},
  },
};
use serde::{Serialize, Deserialize};

#[derive(Debug, BitFlags, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum StatusInfoEnum {
  Disposed = 0b0001,
  Unregistered = 0b0010,
  Filtered = 0b0100,
}

/// StatusInfo is a 4 octet array -> 9.6.3.9 StatusInfo_t
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StatusInfo {
  em: [u8; 3],
  si: BitFlags<StatusInfoEnum>,
}

impl StatusInfo {
  pub fn empty() -> StatusInfo {
    StatusInfo {
      em: [0; 3],
      si: BitFlags::empty(),
    }
  }

  pub fn contains(&self, sie: StatusInfoEnum) -> bool {
    self.si.contains(sie)
  }

  pub fn change_kind(&self) -> ChangeKind {
    if self.contains(StatusInfoEnum::Disposed) {
      // DISPOSED is strongest
      ChangeKind::NOT_ALIVE_DISPOSED
    } else if self.contains(StatusInfoEnum::Unregistered) {
      // Checking unregistered second
      ChangeKind::NOT_ALIVE_UNREGISTERED
    } else {
      // Even if filtered is set it is still alive
      ChangeKind::ALIVE
    }
  }

  pub fn into_cdr_bytes<BO: ByteOrder>(
    &self,
  ) -> Result<Vec<u8>, crate::serialization::error::Error> {
    to_bytes::<StatusInfo, BO>(&self)
  }

  pub fn from_cdr_bytes(
    bytes: &Vec<u8>,
    representation_id: RepresentationIdentifier,
  ) -> Result<StatusInfo, crate::serialization::error::Error> {
    CDRDeserializerAdapter::from_bytes(bytes, representation_id)
  }
}

/// RTPS KeyHash -> 9.6.3.8 KeyHash
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub struct KeyHash {
  key: u128,
}

impl KeyHash {
  pub fn empty() -> KeyHash {
    KeyHash { key: 0 }
  }

  pub fn value(&self) -> u128 {
    self.key
  }

  pub fn into_cdr_bytes<BO: ByteOrder>(
    &self,
  ) -> Result<Vec<u8>, crate::serialization::error::Error> {
    to_bytes::<KeyHash, BO>(&self)
  }

  pub fn from_cdr_bytes(
    bytes: &Vec<u8>,
    representation_id: RepresentationIdentifier,
  ) -> Result<KeyHash, crate::serialization::error::Error> {
    let mut rdr = Cursor::new(bytes);
    let key = match representation_id {
      RepresentationIdentifier::CDR_BE | RepresentationIdentifier::PL_CDR_BE => rdr
        .read_u128::<BigEndian>()
        .map_err(|e| crate::serialization::error::Error::IOError(e))?,
      RepresentationIdentifier::CDR_LE | RepresentationIdentifier::PL_CDR_LE => rdr
        .read_u128::<LittleEndian>()
        .map_err(|e| crate::serialization::error::Error::IOError(e))?,
      _ => 0,
    };
    Ok(KeyHash { key })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inline_qos_status_info() {
    // Little endian
    let si_bytes = StatusInfo {
      em: [0; 3],
      si: StatusInfoEnum::Disposed | StatusInfoEnum::Unregistered,
    }
    .into_cdr_bytes::<LittleEndian>()
    .unwrap();

    let bytes: Vec<u8> = vec![0x00, 0x00, 0x00, 0x03];
    assert_eq!(si_bytes, bytes);

    let status_info = StatusInfo::from_cdr_bytes(&bytes, RepresentationIdentifier::CDR_LE).unwrap();
    assert_eq!(
      status_info,
      StatusInfo {
        em: [0; 3],
        si: StatusInfoEnum::Disposed | StatusInfoEnum::Unregistered
      }
    );

    // Big endian
    let si_bytes = StatusInfo {
      em: [0; 3],
      si: StatusInfoEnum::Disposed | StatusInfoEnum::Unregistered,
    }
    .into_cdr_bytes::<BigEndian>()
    .unwrap();

    let bytes: Vec<u8> = vec![0x00, 0x00, 0x00, 0x03];
    assert_eq!(si_bytes, bytes);

    let status_info = StatusInfo::from_cdr_bytes(&bytes, RepresentationIdentifier::CDR_BE).unwrap();
    assert_eq!(
      status_info,
      StatusInfo {
        em: [0; 3],
        si: StatusInfoEnum::Disposed | StatusInfoEnum::Unregistered
      }
    );
  }

  #[test]
  fn inline_qos_key_hash() {
    // Little endian
    let hbytes = KeyHash { key: 1 }.into_cdr_bytes::<LittleEndian>().unwrap();

    let bytes: Vec<u8> = vec![
      0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00,
    ];
    assert_eq!(hbytes, bytes);

    let key_hash = KeyHash::from_cdr_bytes(&bytes, RepresentationIdentifier::CDR_LE).unwrap();
    assert_eq!(KeyHash { key: 1 }, key_hash);

    // Big endian
    let hbytes = KeyHash { key: 1 }.into_cdr_bytes::<BigEndian>().unwrap();
    let bytes: Vec<u8> = vec![
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x01,
    ];
    assert_eq!(hbytes, bytes);
    let key_hash = KeyHash::from_cdr_bytes(&bytes, RepresentationIdentifier::CDR_BE).unwrap();
    assert_eq!(KeyHash { key: 1 }, key_hash);
  }
}
