use glam::{Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

use crate::*;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Color(Vec4);

impl ValueType for Color {
  fn binding(&self) -> Option<&str> {
    Some("Color")
  }

  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec4(self.0)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec4(v) => {
        *self = Self(v);
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec4 (Color) got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
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
  fn binding(&self) -> Option<&str> {
    Some("ColorRGB")
  }

  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec3(v) => {
        *self = Self(v);
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec3 (ColorRGB) got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
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
  fn binding(&self) -> Option<&str> {
    Some("Bitangent")
  }

  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec3(v) => {
        *self = Self(v);
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec3 (Bitangent) got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
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
  fn binding(&self) -> Option<&str> {
    Some("Tangent")
  }

  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec3(v) => {
        *self = Self(v);
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec3 (Tangent) got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
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
  fn binding(&self) -> Option<&str> {
    Some("Normal")
  }

  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec3(v) => {
        *self = Self(v);
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec3 (Normal) got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
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
  fn binding(&self) -> Option<&str> {
    Some("Position")
  }

  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec3(self.0)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec3(v) => {
        *self = Self(v);
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec3 (Position) got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.0.as_mut())
  }
}

impl_enum_parameter_type!(
  #[derive(PartialEq, Eq)]
  pub enum UvChannel {
    UV0,
    UV1,
    UV2,
    UV3,
  }
);

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UV(Vec2, UvChannel);

impl ValueType for UV {
  fn binding(&self) -> Option<&str> {
    match self.1 {
      UvChannel::UV0 => Some("UV0"),
      UvChannel::UV1 => Some("UV1"),
      UvChannel::UV2 => Some("UV2"),
      UvChannel::UV3 => Some("UV3"),
    }
  }

  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec2(self.0)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec2(v) => {
        *self = Self(v, Default::default());
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec2 (UV) got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Vec2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    egui::ComboBox::from_id_source("UV Channel")
      .selected_text(format!("{:?}", self.1))
      .show_ui(ui, |ui| {
        ui.selectable_value(&mut self.1, UvChannel::UV0, "UV0");
        ui.selectable_value(&mut self.1, UvChannel::UV1, "UV1");
        ui.selectable_value(&mut self.1, UvChannel::UV2, "UV2");
        ui.selectable_value(&mut self.1, UvChannel::UV3, "UV3");
      })
      .response
  }
}
