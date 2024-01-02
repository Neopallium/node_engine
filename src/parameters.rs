use anyhow::Result;

use crate::*;

impl_enum_parameter_type!(
  #[derive(PartialEq, Eq)]
  pub enum CoordSpace {
    Object,
    View,
    World,
    Tangent,
  }
);

impl_enum_parameter_type!(
  #[derive(PartialEq, Eq)]
  pub enum UvChannel {
    UV0,
    UV1,
    UV2,
    UV3,
  }
);
