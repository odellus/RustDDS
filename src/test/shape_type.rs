use crate::dds::traits::key::{Keyed};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct ShapeType {
  a: i32,
}

impl Keyed for ShapeType {
  type K = i32;
  fn get_key(&self) -> Self::K {
    self.a
  }
}
