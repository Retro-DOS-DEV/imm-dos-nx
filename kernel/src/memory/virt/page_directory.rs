use super::super::address::PhysicalAddress;

#[cfg(not(test))]
pub fn set_current_pagedir(addr: PhysicalAddress) {
  crate::x86::registers::set_cr3(addr.as_u32());
}

#[cfg(not(test))]
pub fn get_current_pagedir() -> PhysicalAddress {
  let cr3 = crate::x86::registers::get_cr3();
  PhysicalAddress::new(cr3 as usize)
}

// For testing:
#[cfg(test)]
static mut MOCK_CR3: u32 = 0;

#[cfg(test)]
pub fn set_current_pagedir(addr: PhysicalAddress) {
  unsafe {
    MOCK_CR3 = addr.as_u32();
  }
}

#[cfg(test)]
pub fn get_current_pagedir() -> PhysicalAddress {
  let cr3 = unsafe { MOCK_CR3 };
  PhysicalAddress::new(cr3 as usize)
}