use anyhow::{anyhow, Result};

use crate::*;

impl ValueType for Texture2DHandle {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Texture2D(self.clone())
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Texture2D(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Texture2D got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Texture2D
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.label("Texture2D");
    false
  }
}

impl ValueType for Texture2DArrayHandle {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Texture2DArray(self.clone())
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Texture2DArray(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Texture2DArray got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Texture2DArray
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.label("Texture2DArray");
    false
  }
}

impl ValueType for Texture3DHandle {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Texture3D(self.clone())
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Texture3D(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Texture3D got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Texture3D
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.label("Texture3D");
    false
  }
}

impl ValueType for CubemapHandle {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    Value::Cubemap(self.clone())
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Cubemap(v) => {
        *self = v;
        Ok(())
      }
      _ => Err(anyhow!("Expected a Cubemap got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::Cubemap
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.label("Cubemap");
    false
  }
}
