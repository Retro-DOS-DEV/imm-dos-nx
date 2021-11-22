use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use super::driver::DeviceDriverType;

/// Associates a unique device name with the device number
pub struct DeviceNumberByName {
  pub name: Box<str>,
  pub number: usize,
}

impl DeviceNumberByName {
  pub fn matches_name(&self, name: &str) -> bool {
    *self.name == *name
  }
}

pub struct InstalledDevices {
  drivers: Vec<Arc<Box<DeviceDriverType>>>,
  device_names: Vec<DeviceNumberByName>, 
}

impl InstalledDevices {
  pub const fn new() -> Self {
    Self {
      drivers: Vec::new(),
      device_names: Vec::new(),
    }
  }

  /// Get a reference to a device driver, given its device number
  pub fn get_device(&self, driver_number: usize) -> Option<&Arc<Box<DeviceDriverType>>> {
    if driver_number > 0 {
      self.drivers.get(driver_number - 1)
    } else {
      None
    }
  }

  /// Look up a device number by its name
  pub fn get_device_number_by_name(&self, seek: &str) -> Option<usize> {
    self.device_names
      .iter()
      .find_map(|by_name| if by_name.matches_name(seek) { Some(by_name.number) } else { None })
  }

  pub fn get_device_by_name(&self, name: &str) -> Option<&Arc<Box<DeviceDriverType>>> {
    let number = self.get_device_number_by_name(name)?;
    self.get_device(number)
  }

  pub fn get_device_name(&self, driver_number: usize) -> Option<&Box<str>> {
    self.device_names.get(driver_number).map(|by_name| &by_name.name)
  }

  pub fn register_driver(&mut self, name: &str, driver: Arc<Box<DeviceDriverType>>) -> usize {
    self.drivers.push(driver);
    let number = self.drivers.len();
    self.device_names.push(
      DeviceNumberByName {
        name: alloc::string::String::from(name).into_boxed_str(),
        number,
      },
    );
    number
  }
}
