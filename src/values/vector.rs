use glam::{Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

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

  let node_style = NodeStyle::get(ui);
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
                egui::DragValue::new(val).speed(0.1).clamp_range(min..=max)
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
      Value::I32(v) => {
        *self = (v as f32, 0.).into();
        Ok(())
      }
      Value::U32(v) => {
        *self = (v as f32, 0.).into();
        Ok(())
      }
      Value::F32(v) => {
        *self = (v, 0.).into();
        Ok(())
      }
      Value::Vec2(v) => {
        *self = v;
        Ok(())
      }
      Value::Vec3(v) => {
        *self = (v.x, v.y).into();
        Ok(())
      }
      Value::Vec4(v) => {
        *self = (v.x, v.y).into();
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
      Value::I32(v) => {
        *self = (v as f32, 0., 0.).into();
        Ok(())
      }
      Value::U32(v) => {
        *self = (v as f32, 0., 0.).into();
        Ok(())
      }
      Value::F32(v) => {
        *self = (v, 0., 0.).into();
        Ok(())
      }
      Value::Vec2(v) => {
        *self = (v, 0.).into();
        Ok(())
      }
      Value::Vec3(v) => {
        *self = v;
        Ok(())
      }
      Value::Vec4(v) => {
        *self = (v.x, v.y, v.z).into();
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
      Value::I32(v) => {
        *self = (v as f32, 0., 0., 1.).into();
        Ok(())
      }
      Value::U32(v) => {
        *self = (v as f32, 0., 0., 1.).into();
        Ok(())
      }
      Value::F32(v) => {
        *self = (v, 0., 0., 1.).into();
        Ok(())
      }
      Value::Vec2(v) => {
        *self = (v, 0., 1.).into();
        Ok(())
      }
      Value::Vec3(v) => {
        *self = (v, 1.).into();
        Ok(())
      }
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

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum DynamicSize {
  #[default]
  D1,
  D2,
  D3,
  D4,
}

impl DynamicSize {
  pub const fn len(&self) -> usize {
    match self {
      Self::D1 => 1,
      Self::D2 => 2,
      Self::D3 => 3,
      Self::D4 => 4,
    }
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DynamicVector {
  data: Vec4,
  size: DynamicSize,
}

impl ValueType for DynamicVector {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    match self.size {
      DynamicSize::D1 => Value::F32(self.data.x),
      DynamicSize::D2 => Value::Vec2(Vec2::from_slice(self.data.as_ref())),
      DynamicSize::D3 => Value::Vec3(Vec3::from_slice(self.data.as_ref())),
      DynamicSize::D4 => Value::Vec4(self.data),
    }
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::I32(v) => {
        *self = Self {
          data: (v as f32, 0., 0., 1.).into(),
          size: DynamicSize::D1,
        };
        Ok(())
      }
      Value::U32(v) => {
        *self = Self {
          data: (v as f32, 0., 0., 1.).into(),
          size: DynamicSize::D1,
        };
        Ok(())
      }
      Value::F32(v) => {
        *self = Self {
          data: (v, 0., 0., 1.).into(),
          size: DynamicSize::D1,
        };
        Ok(())
      }
      Value::Vec2(v) => {
        *self = Self {
          data: (v, 0., 1.).into(),
          size: DynamicSize::D2,
        };
        Ok(())
      }
      Value::Vec3(v) => {
        *self = Self {
          data: (v, 1.).into(),
          size: DynamicSize::D3,
        };
        Ok(())
      }
      Value::Vec4(v) => {
        *self = Self {
          data: v,
          size: DynamicSize::D4,
        };
        Ok(())
      }
      _ => Err(anyhow!("Expected a Dynamic Vector got: {value:?}")),
    }
  }

  fn compile(&self) -> Result<CompiledValue> {
    self.to_value().compile()
  }

  fn data_type(&self) -> DataType {
    DataType::DynamicVector
  }

  fn is_dynamic(&self) -> bool {
    true
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    let len = self.size.len();
    vector_ui(ui, &mut self.data.as_mut()[0..len])
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Dynamic(DynamicVector);

impl ValueType for Dynamic {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    self.0.to_value()
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    self.0.set_value(value)
  }

  fn compile(&self) -> Result<CompiledValue> {
    self.to_value().compile()
  }

  fn data_type(&self) -> DataType {
    DataType::Dynamic
  }

  fn is_dynamic(&self) -> bool {
    true
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    self.0.ui(ui)
  }
}
