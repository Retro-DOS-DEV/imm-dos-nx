use crate::memory::address::PhysicalAddress;
use crate::x86::io::Port;
use spin::{Mutex, MutexGuard};

/**
 * Interface with old-school ISA DMA. There are only two chips present in the
 * system, and each enforces locks to ensure there are no conflicts between
 * drivers when setting up addresses, configuration, etc.
 */
pub struct DMAController {
  registers: DMARegisters,
  // The mask_base serves as the starting point for masking, and the lock around
  // it is used to ensure only one thread can edit registers at a time
  mask_base: Mutex<u8>,
}

impl DMAController {
  pub fn get_channel(&self, channel: u8) -> DMAChannel {
    let lock = self.mask_base.lock();
    unsafe {
      self.registers.flip_flop_reset.write_u8(0xff);
    }
    let channel_mask = *lock | (1 << channel as usize);
    DMAChannel::new(lock, self.registers.get_channel_registers(channel))
  }

  pub const fn low_channels() -> DMAController {
    DMAController {
      registers: DMARegisters::low_channels(),
      mask_base: Mutex::new(1),
    }
  }
}

struct DMARegisters {
  start_address_1: Port,
  count_register_1: Port,
  page_1: Port,
  start_address_2: Port,
  count_register_2: Port,
  page_2: Port,
  start_address_3: Port,
  count_register_3: Port,
  page_3: Port,
  mode: Port,
  flip_flop_reset: Port,
  pub multi_channel_mask: Port,
}

impl DMARegisters {
  pub const fn low_channels() -> DMARegisters {
    DMARegisters {
      start_address_1: Port::new(0x02),
      count_register_1: Port::new(0x03),
      page_1: Port::new(0x83),
      start_address_2: Port::new(0x04),
      count_register_2: Port::new(0x05),
      page_2: Port::new(0x81),
      start_address_3: Port::new(0x06),
      count_register_3: Port::new(0x07),
      page_3: Port::new(0x82),
      mode: Port::new(0x0b),
      flip_flop_reset: Port::new(0x0c),
      multi_channel_mask: Port::new(0x0f)
    }
  }

  pub fn get_channel_registers(&self, channel: u8) -> ChannelRegisters {
    let (start_address, count_register, page) = match channel {
      1 => (self.start_address_1, self.count_register_1, self.page_1),
      2 => (self.start_address_2, self.count_register_2, self.page_2),
      3 => (self.start_address_3, self.count_register_3, self.page_3),
      _ => panic!("invalid channel"),
    };
    ChannelRegisters {
      start_address,
      count_register,
      page,
      mode: self.mode,
      flip_flop_reset: self.flip_flop_reset,
      multi_channel_mask: self.multi_channel_mask,
    }
  }
}

pub struct ChannelRegisters {
  pub start_address: Port,
  pub count_register: Port,
  pub page: Port,
  pub mode: Port,
  pub flip_flop_reset: Port,
  pub multi_channel_mask: Port,
}

impl ChannelRegisters {
  pub fn set_address(&self, addr: PhysicalAddress) {
    let addr_32 = addr.as_u32();
    let addr_low = (addr_32 & 0xff) as u8;
    let addr_mid = ((addr_32 >> 8) & 0xff) as u8;
    let addr_high = ((addr_32 >> 16) & 0xff) as u8;

    unsafe {
      self.flip_flop_reset.write_u8(0xff);
      self.start_address.write_u8(addr_low);
      self.start_address.write_u8(addr_mid);
      self.page.write_u8(addr_high);
    }
  }

  pub fn set_count(&self, count: usize) {
    let count_low = (count & 0xff) as u8;
    let count_high = ((count >> 8) & 0xff) as u8;

    unsafe {
      self.flip_flop_reset.write_u8(0xff);
      self.count_register.write_u8(count_low);
      self.count_register.write_u8(count_high);
    }
  }
}

pub struct DMAChannel<'a> {
  mask_base: MutexGuard<'a, u8>,
  registers: ChannelRegisters,
}

impl<'a> DMAChannel<'a> {
  pub fn new(mask_base: MutexGuard<'a, u8>, registers: ChannelRegisters) -> DMAChannel {
    DMAChannel {
      mask_base,
      registers,
    }
  }

  pub fn set_address(&self, addr: PhysicalAddress) {
    self.registers.set_address(addr);
  }

  pub fn set_count(&self, count: usize) {
    self.registers.set_count(count);
  }

  pub fn set_mode(&self, mode: u8) {
    unsafe {
      self.registers.mode.write_u8(mode);
    }
  }
}

/**
 * When DMAChannel passes out of scope, we want to unmask the channel
 */
impl<'a> Drop for DMAChannel<'a> {
  fn drop(&mut self) {
    unsafe {
      self.registers.multi_channel_mask.write_u8(*self.mask_base);
    }
  }
}

pub struct DMA {
  low: DMAController,
  //high: DMAController,
}

impl DMA {
  pub const fn new() -> DMA {
    DMA {
      low: DMAController::low_channels(),
    }
  }

  pub fn get_channel(&self, channel: u8) -> DMAChannel {
    let source = &self.low;
    source.get_channel(channel % 4)
  }
}
