use super::super::address::PhysicalAddress;

/**
 * We can use the same struct for the Page Directory and each Page Table.
 * Entries are 32-bit values with the following layout:
 * 31       11      9       0
 * | ADDRESS | FREE | FLAGS |
 */
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
  pub const fn new() -> PageTableEntry {
    PageTableEntry(0)
  }

  pub fn get_address(&self) -> PhysicalAddress {
    PhysicalAddress::new((self.0 & 0xfffff000) as usize)
  }

  pub fn set_address(&mut self, addr: PhysicalAddress) {
    let addr_bits = addr.as_u32() & 0xfffff000;
    self.0 = (self.0 & 0xfff) | addr_bits;
  }

  pub fn has_been_accessed(&self) -> bool {
    self.0 & 0x20 == 0x20
  }

  pub fn set_user_access(&mut self) {
    self.0 |= 0x4;
  }

  pub fn clear_user_access(&mut self) {
    self.0 &= !0x4;
  }

  pub fn is_user_access_granted(&self) -> bool {
    self.0 & 0x4 == 0x4
  }

  pub fn set_write_access(&mut self) {
    self.0 |= 0x2;
  }

  pub fn clear_write_access(&mut self) {
    self.0 &= !0x2;
  }

  pub fn is_write_access_granted(&self) -> bool {
    self.0 & 0x2 == 0x2
  }

  pub fn set_present(&mut self) {
    self.0 |= 1;
  }

  pub fn clear_present(&mut self) {
    self.0 &= !1;
  }

  pub fn is_present(&self) -> bool {
    self.0 & 0x1 == 0x1
  }
}
