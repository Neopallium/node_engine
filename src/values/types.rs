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

pub trait ValueType: Clone {
  fn has_binding(&self) -> Option<&str> {
    None
  }

  fn to_value(&self) -> Value;

  fn from_value(value: Value) -> Result<Self>;

  fn data_type() -> DataType;

  fn eval(&self, _graph: &NodeGraph, _execution: &mut NodeGraphExecution) -> Result<Self> {
    Ok(self.clone())
  }

  fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile) -> Result<String> {
    compile.compile_value(&self.to_value())
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.label("No UI for type.")
  }
}

impl ValueType for i32 {
  fn to_value(&self) -> Value {
    Value::I32(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::I32(v) => Ok(v),
      _ => Err(anyhow!("Expected a I32 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::I32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.add(egui::DragValue::new(self).speed(1))
  }
}

impl ValueType for u32 {
  fn to_value(&self) -> Value {
    Value::U32(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::U32(v) => Ok(v),
      _ => Err(anyhow!("Expected a U32 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::U32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.add(egui::DragValue::new(self).speed(1))
  }
}

impl ValueType for f32 {
  fn to_value(&self) -> Value {
    Value::F32(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::F32(v) => Ok(v),
      _ => Err(anyhow!("Expected a F32 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::F32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.add(egui::DragValue::new(self).speed(0.1))
  }
}

#[cfg(feature = "egui")]
const COLUMNS: [&str; 4] = ["X", "Y", "Z", "W"];

#[cfg(feature = "egui")]
pub(crate) fn f32_table_ui(
  ui: &mut egui::Ui,
  columns: &[&str],
  rows: usize,
  values: &mut [f32],
) -> egui::Response {
  use egui_extras::{Column, TableBuilder};

  let node_style = ui.node_style();
  let mut resp = ui.interact(ui.min_rect(), ui.id(), egui::Sense::click());
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
              if values[col * rows + row_index].ui(ui).changed() {
                resp.mark_changed();
              }
            });
          }
        });
      });
  });
  resp
}

#[cfg(feature = "egui")]
pub(crate) fn vector_ui(ui: &mut egui::Ui, values: &mut [f32]) -> egui::Response {
  let len = values.len();
  f32_table_ui(ui, &COLUMNS[0..len], 1, values)
}

#[cfg(feature = "egui")]
pub(crate) fn matrix_ui(ui: &mut egui::Ui, dim: usize, values: &mut [f32]) -> egui::Response {
  f32_table_ui(ui, &COLUMNS[0..dim], dim, values)
}

impl ValueType for Vec2 {
  fn to_value(&self) -> Value {
    Value::Vec2(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec2(v) => Ok(v),
      _ => Err(anyhow!("Expected a Vec2 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Vec3 {
  fn to_value(&self) -> Value {
    Value::Vec3(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec3(v) => Ok(v),
      _ => Err(anyhow!("Expected a Vec3 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Vec4 {
  fn to_value(&self) -> Value {
    Value::Vec4(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec4(v) => Ok(v),
      _ => Err(anyhow!("Expected a Vec4 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec4
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Mat2 {
  fn to_value(&self) -> Value {
    Value::Mat2(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Mat2(v) => Ok(v),
      _ => Err(anyhow!("Expected a Mat2 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Mat2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    matrix_ui(ui, 2, &mut self.as_mut()[..])
  }
}

impl ValueType for Mat3 {
  fn to_value(&self) -> Value {
    Value::Mat3(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Mat3(v) => Ok(v),
      _ => Err(anyhow!("Expected a Mat3 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Mat3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    matrix_ui(ui, 3, &mut self.as_mut()[..])
  }
}

impl ValueType for Mat4 {
  fn to_value(&self) -> Value {
    Value::Mat4(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Mat4(v) => Ok(v),
      _ => Err(anyhow!("Expected a Mat4 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Mat4
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    matrix_ui(ui, 4, &mut self.as_mut()[..])
  }
}
