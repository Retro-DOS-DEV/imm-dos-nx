use crate::memory::address::{PhysicalAddress, VirtualAddress};

/// A memory backup is used to track memory-mapped devices (like video RAM) that
/// are backed by an in-memory copy. When a process using the device is
/// "inactive," it can update the backup copy and wait until that backup is
/// restored to the device.
#[derive(Copy, Clone)]
pub struct MemoryBackup {
  /// The location of the actual hardware on the memory bus
  physical_address: PhysicalAddress,
  /// The physical address of the memory used to back it up, useful for making
  /// new page mappings
  buffer_frame: PhysicalAddress,
  /// The virtual address (in kernel space) for the backup page. When the page
  /// is allocated, it must at least be accessible from the kernel.
  pub mapped_to: VirtualAddress,
}

impl MemoryBackup {
  pub fn allocate(physical_address: PhysicalAddress) -> Self {
    use crate::memory::virt::page_directory::{CurrentPageDirectory, PageDirectory, PermissionFlags};
    use crate::task::memory::MMapBacking;

    let frame = crate::memory::physical::allocate_frame().unwrap();
    let buffer_frame = frame.get_address();

    let mapped_to = crate::task::memory::kernel_mmap(None, 0x1000, MMapBacking::Direct(buffer_frame)).unwrap();
    let pagedir = CurrentPageDirectory::get();
    pagedir.map(frame, mapped_to, PermissionFlags::empty());

    Self {
      physical_address,
      buffer_frame,
      mapped_to,
    }
  }

  pub fn get_buffer_physical_address(&self) -> PhysicalAddress {
    self.buffer_frame
  }

  pub unsafe fn copy_to_buffer(&self) {
    self.get_backup_buffer().copy_from_slice(self.get_device_buffer())
  }

  pub unsafe fn copy_from_buffer(&self) {
    self.get_device_buffer().copy_from_slice(self.get_backup_buffer())
  }

  unsafe fn get_device_buffer(&self) -> &mut [u8] {
    let ptr = self.get_kernel_location().as_usize() as *mut u8;
    core::slice::from_raw_parts_mut(ptr, 0x1000)
  }

  unsafe fn get_backup_buffer(&self) -> &mut [u8] {
    let ptr = self.mapped_to.as_usize() as *mut u8;
    core::slice::from_raw_parts_mut(ptr, 0x1000)
  }

  /// Determine where this physical memory is accessible from the kernel
  fn get_kernel_location(&self) -> VirtualAddress {
    VirtualAddress::new(
      self.physical_address.as_usize() + 0xc0000000
    )
  }
}