use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTypeClass {
  Scalar,
  Vector,
  Matrix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataType {
  I32,
  U32,
  F32,
  Vec2,
  Vec3,
  Vec4,
  Mat2,
  Mat3,
  Mat4,
}

impl DataType {
  pub fn class(&self) -> DataTypeClass {
    match self {
      Self::I32 => DataTypeClass::Scalar,
      Self::U32 => DataTypeClass::Scalar,
      Self::F32 => DataTypeClass::Scalar,
      Self::Vec2 => DataTypeClass::Vector,
      Self::Vec3 => DataTypeClass::Vector,
      Self::Vec4 => DataTypeClass::Vector,
      Self::Mat2 => DataTypeClass::Matrix,
      Self::Mat3 => DataTypeClass::Matrix,
      Self::Mat4 => DataTypeClass::Matrix,
    }
  }

  pub fn default_value(&self) -> Value {
    match self {
      Self::I32 => Value::I32(Default::default()),
      Self::U32 => Value::U32(Default::default()),
      Self::F32 => Value::F32(Default::default()),
      Self::Vec2 => Value::Vec2(Default::default()),
      Self::Vec3 => Value::Vec3(Default::default()),
      Self::Vec4 => Value::Vec4(Default::default()),
      Self::Mat2 => Value::Mat2(Default::default()),
      Self::Mat3 => Value::Mat3(Default::default()),
      Self::Mat4 => Value::Mat4(Default::default()),
    }
  }

  #[cfg(feature = "egui")]
  pub fn color(&self) -> egui::Color32 {
    match self {
      Self::I32 => egui::Color32::LIGHT_BLUE,
      Self::U32 => egui::Color32::LIGHT_BLUE,
      Self::F32 => egui::Color32::LIGHT_BLUE,
      Self::Vec2 => egui::Color32::GREEN,
      Self::Vec3 => egui::Color32::YELLOW,
      Self::Vec4 => egui::Color32::LIGHT_RED,
      Self::Mat2 => egui::Color32::BLUE,
      Self::Mat3 => egui::Color32::BLUE,
      Self::Mat4 => egui::Color32::BLUE,
    }
  }

  pub fn is_compatible(&self, other: &DataType) -> bool {
    if self == other {
      // Same type is compatible.
      true
    } else if self.class() == other.class() {
      // Same class is compatible.
      true
    } else {
      match (self.class(), other.class()) {
        (DataTypeClass::Scalar, DataTypeClass::Vector)
        | (DataTypeClass::Vector, DataTypeClass::Scalar) => true,
        _ => false,
      }
    }
  }
}

pub trait ValueType: core::fmt::Debug {
  fn clone_value(&self) -> Box<dyn ValueType>;

  fn set_value(&mut self, value: Value) -> Result<()>;

  fn to_value(&self) -> Value;

  fn data_type(&self) -> DataType;

  fn binding(&self) -> Option<&str> {
    None
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.label("No UI for type.");
    false
  }
}

impl Clone for Box<dyn ValueType> {
  fn clone(&self) -> Self {
    self.clone_value()
  }
}

impl ValueType for i32 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::I32(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::I32(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a I32 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::I32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.add(egui::DragValue::new(self).speed(1)).changed()
  }
}

impl ValueType for u32 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::U32(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::U32(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a U32 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::U32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.add(egui::DragValue::new(self).speed(1)).changed()
  }
}

impl ValueType for f32 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::F32(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::F32(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a F32 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::F32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.add(egui::DragValue::new(self).speed(0.1)).changed()
  }
}

#[cfg(feature = "egui")]
const COLUMNS: [&str; 4] = ["X", "Y", "Z", "W"];
#[cfg(feature = "egui")]
const COLOR_COLUMNS: [&str; 4] = ["R", "G", "B", "A"];

#[cfg(feature = "egui")]
pub(crate) fn f32_table_ui(
  ui: &mut egui::Ui,
  columns: &[&str],
  rows: usize,
  values: &mut [f32],
  range: Option<(f32, f32)>,
) -> bool {
  use egui_extras::{Column, TableBuilder};

  let node_style = ui.node_style();
  let mut changed = false;
  let height = 20.0 * node_style.zoom;
  let width = 40.0 * node_style.zoom;

  // Allocate space for the table.
  ui.set_min_height(height * (rows + 1) as f32);

  // Make sure the layout is in vertical mode.
  ui.vertical(|ui| {
    TableBuilder::new(ui)
      .columns(Column::exact(width), columns.len())
      .vscroll(false)
      .header(height, |mut header| {
        for col in columns {
          header.col(|ui| {
            ui.heading(*col);
          });
        }
      })
      .body(|body| {
        body.rows(height, rows, |row_index, mut row| {
          for col in 0..columns.len() {
            row.col(|ui| {
              let val = &mut values[col * rows + row_index];
              let drag = if let Some((min, max)) = range {
                egui::DragValue::new(val)
                  .speed(0.1)
                  .clamp_range(min..=max)
              } else {
                egui::DragValue::new(val).speed(0.1)
              };
              if ui.add(drag).changed() {
                changed = true;
              }
            });
          }
        });
      });
  });
  changed
}

#[cfg(feature = "egui")]
pub(crate) fn vector_ui(ui: &mut egui::Ui, values: &mut [f32]) -> bool {
  let len = values.len();
  f32_table_ui(ui, &COLUMNS[0..len], 1, values, None)
}

#[cfg(feature = "egui")]
pub(crate) fn color_ui(ui: &mut egui::Ui, values: &mut [f32]) -> bool {
  let len = values.len();
  f32_table_ui(ui, &COLOR_COLUMNS[0..len], 1, values, Some((0., 1.)))
}

#[cfg(feature = "egui")]
pub(crate) fn matrix_ui(ui: &mut egui::Ui, dim: usize, values: &mut [f32]) -> bool {
  f32_table_ui(ui, &COLUMNS[0..dim], dim, values, None)
}

impl ValueType for Vec2 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec2(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec2(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec2 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Vec2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Vec3 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec3(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec3(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec3 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Vec4 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Vec4(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Vec4(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Vec4 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Vec4
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Mat2 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Mat2(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Mat2(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Mat2 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Mat2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    matrix_ui(ui, 2, &mut self.as_mut()[..])
  }
}

impl ValueType for Mat3 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Mat3(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Mat3(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Mat3 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Mat3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    matrix_ui(ui, 3, &mut self.as_mut()[..])
  }
}

impl ValueType for Mat4 {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Mat4(*self)
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Mat4(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Mat4 got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Mat4
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    matrix_ui(ui, 4, &mut self.as_mut()[..])
  }
}
