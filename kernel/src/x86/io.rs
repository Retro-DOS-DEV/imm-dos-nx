pub struct Port {
  number: u16,
}

impl Port {
  pub const fn new(number: u16) -> Port {
    Port {
      number,
    }
  }

  pub unsafe fn write_u8(&self, value: u8) {
    outb(self.number, value);
  }

  pub unsafe fn read_u8(&self) -> u8 {
    inb(self.number)
  }

  pub unsafe fn write_u16(&self, value: u16) {
    outw(self.number, value);
  }

  pub unsafe fn read_u16(&self) -> u16 {
    inw(self.number)
  }

  pub unsafe fn write_u32(&self, value: u32) {
    outl(self.number, value);
  }

  pub unsafe fn read_32(&self) -> u32 {
    inl(self.number)
  }
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
  let value: u8;
  llvm_asm!("in al, dx" : "={al}"(value) : "{dx}"(port) : "ax", "dx" : "intel", "volatile");
  value
}

#[inline]
pub unsafe fn outb(port: u16, value: u8) {
  llvm_asm!("out dx, al" :: "{al}"(value), "{dx}"(port) : "ax", "dx" : "intel", "volatile");
}

#[inline]
pub unsafe fn inw(port: u16) -> u16 {
  let value: u16;
  llvm_asm!("in ax, dx" : "={ax}"(value) : "{dx}"(port) : "ax", "dx" : "intel", "volatile");
  value
}

#[inline]
pub unsafe fn outw(port: u16, value: u16) {
  llvm_asm!("out dx, ax" :: "{ax}"(value), "{dx}"(port) : "ax", "dx" : "intel", "volatile");
}

#[inline]
pub unsafe fn inl(port: u16) -> u32 {
  let value: u32;
  llvm_asm!("in eax, dx" : "={eax}"(value) : "{dx}"(port) : "eax", "dx" : "intel", "volatile");
  value
}

#[inline]
pub unsafe fn outl(port: u16, value: u32) {
  llvm_asm!("out dx, eax" :: "{eax}"(value), "{dx}"(port) : "eax", "dx" : "intel", "volatile");
}