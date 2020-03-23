#[repr(C, packed)]
pub struct StringPtr {
  pub addr: usize,
  pub length: usize,
}

impl StringPtr {
  pub fn from_str(s: &str) -> StringPtr {
    StringPtr {
      addr: s.as_ptr() as usize,
      length: s.len(),
    }
  }

  pub fn get_starting_ptr(&self) -> *const u8 {
    self.addr as *const u8
  }

  pub unsafe fn as_str(&self) -> &'static str {
    let bytes = core::slice::from_raw_parts(self.get_starting_ptr(), self.length);
    core::str::from_utf8_unchecked(bytes)
  }
}