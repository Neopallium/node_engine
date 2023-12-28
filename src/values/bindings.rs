use glam::{Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

use crate::*;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Color(Vec4);

impl ValueType for Color {
  fn to_value(&self) -> Value {
    Value::Vec4(self.0)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec4(v) => Ok(Self(v)),
      _ => Err(anyhow!("Expected a Vec4 (Color) got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec4
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ColorRGB(Vec3);

impl ValueType for ColorRGB {
  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec3(v) => Ok(Self(v)),
      _ => Err(anyhow!("Expected a Vec3 (ColorRGB) got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Bitangent(Vec3);

impl ValueType for Bitangent {
  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec3(v) => Ok(Self(v)),
      _ => Err(anyhow!("Expected a Vec3 (Bitangent) got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Tangent(Vec3);

impl ValueType for Tangent {
  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec3(v) => Ok(Self(v)),
      _ => Err(anyhow!("Expected a Vec3 (Tangent) got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Normal(Vec3);

impl ValueType for Normal {
  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec3(v) => Ok(Self(v)),
      _ => Err(anyhow!("Expected a Vec3 (Normal) got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Position(Vec3);

impl ValueType for Position {
  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec3(v) => Ok(Self(v)),
      _ => Err(anyhow!("Expected a Vec3 (Position) got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum UvChannel {
  #[default]
  UV0,
  UV1,
  UV2,
  UV3,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UV(Vec2, UvChannel);

impl ValueType for UV {
  fn to_value(&self) -> Value {
    Value::Vec2(self.0)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec2(v) => Ok(Self(v, Default::default())),
      _ => Err(anyhow!("Expected a Vec2 (UV) got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}
