use serde::{Serialize /*, Deserialize*/};
use bytes::Bytes;

use crate::{
  dds::traits::key::Keyed,
  structure::{
    inline_qos::{KeyHash, StatusInfo},
  },
};
use crate::messages::submessages::submessage_elements::serialized_payload::RepresentationIdentifier;
use crate::messages::submessages::submessage_elements::serialized_payload::SerializedPayload;
use crate::serialization::cdr_serializer::{to_bytes};
use byteorder::{LittleEndian /*,BigEndian*/};

use crate::structure::guid::EntityId;
use crate::structure::time::Timestamp;
use crate::structure::cache_change::ChangeKind;

// DDSData represets a serialized data sample with metadata
#[derive(Debug, PartialEq, Clone)]
pub struct DDSData {
  source_timestamp: Timestamp,
  pub change_kind: ChangeKind,
  reader_id: EntityId,
  writer_id: EntityId,
  value: Option<SerializedPayload>,
  // needed to identify what instance type (unique key) this change is for 9.6.3.8
  pub value_key_hash: u128,
}

impl DDSData {
  pub fn new(payload: SerializedPayload) -> DDSData {
    DDSData {
      source_timestamp: Timestamp::now(),
      change_kind: ChangeKind::ALIVE,
      reader_id: EntityId::ENTITYID_UNKNOWN,
      writer_id: EntityId::ENTITYID_UNKNOWN,
      value: Some(payload),
      value_key_hash: 0,
    }
  }

  pub fn new_disposed(status_info: Option<StatusInfo>, key_hash: Option<KeyHash>) -> DDSData {
    let change_kind = match status_info {
      Some(i) => i.change_kind(),
      // no change kind/status info means that it's still alive
      None => ChangeKind::ALIVE,
    };

    let value_key_hash = match key_hash {
      Some(v) => v,
      None => KeyHash::empty(),
    };

    DDSData {
      source_timestamp: Timestamp::now(),
      change_kind,
      reader_id: EntityId::ENTITYID_UNKNOWN,
      writer_id: EntityId::ENTITYID_UNKNOWN,
      value: None,
      value_key_hash: value_key_hash.value(),
    }
  }

  // TODO: Rename this method, as it gets confued with the std library "From" trait method.
  pub fn from<D>(data: &D, source_timestamp: Option<Timestamp>) -> DDSData
  where
    D: Keyed + Serialize,
  {
    let value = DDSData::serialize_data(data);

    let ts: Timestamp = match source_timestamp {
      Some(t) => t,
      None => Timestamp::now(),
    };

    let serialized_payload = SerializedPayload::new(RepresentationIdentifier::CDR_LE, value);

    DDSData {
      source_timestamp: ts,
      change_kind: ChangeKind::ALIVE,
      reader_id: EntityId::ENTITYID_UNKNOWN,
      writer_id: EntityId::ENTITYID_UNKNOWN,
      value: Some(serialized_payload),
      value_key_hash: 0,
    }
  }

  pub fn from_dispose<D>(_key: <D as Keyed>::K, source_timestamp: Option<Timestamp>) -> DDSData
  where
    D: Keyed,
  {
    let ts: Timestamp = match source_timestamp {
      Some(t) => t,
      None => Timestamp::now(),
    };

    // TODO: Serialize key

    DDSData {
      source_timestamp: ts,
      change_kind: ChangeKind::NOT_ALIVE_DISPOSED,
      reader_id: EntityId::ENTITYID_UNKNOWN,
      writer_id: EntityId::ENTITYID_UNKNOWN,
      value: None, // TODO: Here we should place the serialized _key_, so that RTPS writer can send the
      // the DATA message indicating dispose
      value_key_hash: 0,
    }
  }

  fn serialize_data<D>(data: &D) -> Vec<u8>
  where
    D: Keyed + Serialize,
  {
    let value = match to_bytes::<D, LittleEndian>(data) {
      Ok(v) => v,
      // TODO: handle error
      _ => Vec::new(),
    };
    value
  }

  pub fn reader_id(&self) -> &EntityId {
    &self.reader_id
  }

  pub fn set_reader_id(&mut self, reader_id: EntityId) {
    self.reader_id = reader_id;
  }

  pub fn writer_id(&self) -> &EntityId {
    &self.writer_id
  }

  pub fn set_writer_id(&mut self, writer_id: EntityId) {
    self.writer_id = writer_id;
  }

  pub fn value(&self) -> Option<SerializedPayload> {
    self.value.clone()
  }

  pub fn data(&self) -> Bytes {
    match &self.value {
      Some(val) => val.value.clone(), // cloning Bytes is cheap
      None => Bytes::new(),
    }
  }
}
