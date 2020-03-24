use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub mod com;
pub mod driver;
pub mod null;
pub mod zero;

pub type DriverType = dyn driver::DeviceDriver + Send + Sync;

pub struct DeviceDrivers {
  drivers: Vec<Arc<Box<DriverType>>>,
}

impl DeviceDrivers {
  pub const fn new() -> DeviceDrivers {
    DeviceDrivers {
      drivers: Vec::new(),
    }
  }

  pub fn get_device(&self, driver_number: usize) -> Option<&Arc<Box<DriverType>>> {
    self.drivers.get(driver_number - 1)
  }

  pub fn register_driver(&mut self, driver: Box<DriverType>) -> usize {
    self.drivers.push(Arc::new(driver));
    self.drivers.len()
  }
}