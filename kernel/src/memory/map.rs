use core::fmt;

pub const REGION_TYPE_FREE: u32 = 1;
pub const REGION_TYPE_RESERVED: u32 = 2;
pub const REGION_TYPE_ACPI_RECOVERABLE: u32 = 3;

#[repr(C, packed)]
pub struct MapEntry {
  pub base: u64,
  pub length: u64,
  pub region_type: u32,
  pub acpi: u32,
}

pub unsafe fn load_entries_at_address(addr: usize) -> &'static [MapEntry] {
  let length = addr as *const usize;
  let first_entry = (addr as *mut u32).offset(1) as *mut MapEntry;
  core::slice::from_raw_parts_mut(first_entry, *length)
}

impl fmt::Debug for MapEntry {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let type_string = match self.region_type {
      REGION_TYPE_FREE => "Free",
      REGION_TYPE_RESERVED => "Reserved",
      REGION_TYPE_ACPI_RECOVERABLE => "ACPI",
      _ => "Unknown",
    };
    let start = self.base;
    let end = self.base + self.length - 1;
    write!(f, "{:#010x}-{:#010x}: {}", start, end, type_string)
  }
}
