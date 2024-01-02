use anyhow::{anyhow, Result};

use crate::*;

impl ValueType for bool {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    if *self {
      Value::U32(1)
    } else {
      Value::U32(0)
    }
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    let val: u32 = match value {
      Value::I32(v) => v as _,
      Value::U32(v) => v,
      Value::F32(v) => v as _,
      Value::Vec2(v) => v.x as _,
      Value::Vec3(v) => v.x as _,
      Value::Vec4(v) => v.x as _,
      _ => return Err(anyhow!("Expected a boolean value got: {value:?}")),
    };
    *self = val == 1;
    Ok(())
  }

  fn data_type(&self) -> DataType {
    DataType::U32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.checkbox(self, "bool").changed()
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
      Value::U32(v) => {
        *self = v as _;
        Ok(())
      }
      Value::F32(v) => {
        *self = v as _;
        Ok(())
      }
      Value::Vec2(v) => {
        *self = v.x as _;
        Ok(())
      }
      Value::Vec3(v) => {
        *self = v.x as _;
        Ok(())
      }
      Value::Vec4(v) => {
        *self = v.x as _;
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
      Value::I32(v) => {
        *self = v as _;
        Ok(())
      }
      Value::U32(v) => {
        *self = v;
        Ok(())
      }
      Value::F32(v) => {
        *self = v as _;
        Ok(())
      }
      Value::Vec2(v) => {
        *self = v.x as _;
        Ok(())
      }
      Value::Vec3(v) => {
        *self = v.x as _;
        Ok(())
      }
      Value::Vec4(v) => {
        *self = v.x as _;
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
      Value::I32(v) => {
        *self = v as _;
        Ok(())
      }
      Value::U32(v) => {
        *self = v as _;
        Ok(())
      }
      Value::F32(v) => {
        *self = v;
        Ok(())
      }
      Value::Vec2(v) => {
        *self = v.x as _;
        Ok(())
      }
      Value::Vec3(v) => {
        *self = v.x as _;
        Ok(())
      }
      Value::Vec4(v) => {
        *self = v.x as _;
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
