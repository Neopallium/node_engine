use glam::{Mat2, Mat3, Mat4, Vec4Swizzles};

use anyhow::{anyhow, Result};

use crate::*;

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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DynamicMatrix {
  data: Mat4,
  size: DynamicSize,
}

impl Default for DynamicMatrix {
  fn default() -> Self {
    Self {
      data: Default::default(),
      size: DynamicSize::D2,
    }
  }
}

impl ValueType for DynamicMatrix {
  fn clone_value(&self) -> Box<dyn ValueType> {
    Box::new(self.clone())
  }

  fn to_value(&self) -> Value {
    match self.size {
      DynamicSize::D1 => {
        // This case shouldn't be possible here.
        panic!("This shouldn't be possible.");
      }
      DynamicSize::D2 => Value::Mat2(Mat2::from_cols(
        self.data.x_axis.xy(),
        self.data.y_axis.xy(),
      )),
      DynamicSize::D3 => Value::Mat3(Mat3::from_mat4(self.data)),
      DynamicSize::D4 => Value::Mat4(self.data),
    }
  }

  fn set_value(&mut self, value: Value) -> Result<()> {
    match value {
      Value::Mat2(v) => {
        *self = Self {
          data: Mat4::from_mat3(Mat3::from_mat2(v)),
          size: DynamicSize::D2,
        };
        Ok(())
      }
      Value::Mat3(v) => {
        *self = Self {
          data: Mat4::from_mat3(v),
          size: DynamicSize::D3,
        };
        Ok(())
      }
      Value::Mat4(v) => {
        *self = Self {
          data: v,
          size: DynamicSize::D4,
        };
        Ok(())
      }
      _ => Err(anyhow!("Expected a Dynamic Matrix got: {value:?}")),
    }
  }

  fn data_type(&self) -> DataType {
    DataType::DynamicMatrix
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    match self.size {
      DynamicSize::D1 => {
        // This case shouldn't be possible here.
        ui.label("INVALID MATRIX");
        false
      }
      DynamicSize::D2 => {
        let mut mat2 = Mat2::from_cols(self.data.x_axis.xy(), self.data.y_axis.xy());
        if matrix_ui(ui, 2, &mut mat2.as_mut()[..]) {
          self.data = Mat4::from_mat3(Mat3::from_mat2(mat2));
          true
        } else {
          false
        }
      }
      DynamicSize::D3 => {
        let mut mat3 = Mat3::from_mat4(self.data);
        if matrix_ui(ui, 3, &mut mat3.as_mut()[..]) {
          self.data = Mat4::from_mat3(mat3);
          true
        } else {
          false
        }
      }
      DynamicSize::D4 => matrix_ui(ui, 4, &mut self.data.as_mut()[..]),
    }
  }
}
