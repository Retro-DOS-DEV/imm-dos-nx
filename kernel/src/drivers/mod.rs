use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub mod blocking;
pub mod com;
pub mod driver;
pub mod floppy;
pub mod keyboard;
pub mod null;
pub mod queue;
pub mod zero;

pub type DeviceName = [u8; 8];
pub type DriverType = dyn driver::DeviceDriver + Send + Sync;

struct DeviceNumberByName(pub DeviceName, pub usize);

pub struct DeviceDrivers {
  drivers: Vec<Arc<Box<DriverType>>>,
  device_names: Vec<DeviceNumberByName>, 
}

impl DeviceDrivers {
  pub const fn new() -> DeviceDrivers {
    DeviceDrivers {
      drivers: Vec::new(),
      device_names: Vec::new(),
    }
  }

  pub fn get_device(&self, driver_number: usize) -> Option<&Arc<Box<DriverType>>> {
    if driver_number > 0 {
      self.drivers.get(driver_number - 1)
    } else {
      None
    }
  }

  pub fn get_device_number_by_name(&self, seek: &DeviceName) -> Option<usize> {
    for entry in self.device_names.iter() {
      if entry.0 == *seek {
        return Some(entry.1);
      }
    }
    None
  }

  pub fn get_device_by_name(&self, name: &DeviceName) -> Option<&Arc<Box<DriverType>>> {
    let number = self.get_device_number_by_name(name)?;
    self.get_device(number)
  }

  pub fn register_driver(&mut self, name: &str, driver: Arc<Box<DriverType>>) -> usize {
    let mut name_array: [u8; 8] = [0x20; 8];
    if name.len() > 8 {
      // Too long
      return 0;
    }
    let name_bytes = name.as_bytes();
    let mut index = 0;
    while index < 8 && index < name_bytes.len() {
      name_array[index] = name_bytes[index];
      index += 1;
    }

    self.drivers.push(driver);
    let index = self.drivers.len();
    self.device_names.push(DeviceNumberByName(name_array, index));
    index
  }
}