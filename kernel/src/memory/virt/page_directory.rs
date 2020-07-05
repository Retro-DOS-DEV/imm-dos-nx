use super::super::address::{PhysicalAddress, VirtualAddress};
use super::super::physical::frame::Frame;
use super::super::physical::allocate_frame;
use super::super::physical::reference_frame_at_address;
use super::page_table::PageTable;
use super::region::{MemoryRegionType, Permissions, VirtualMemoryRegion};

pub struct PermissionFlags(u8);
impl PermissionFlags {
  pub const USER_ACCESS: u8 = 1;
  pub const WRITE_ACCESS: u8 = 2;

  pub fn new(flags: u8) -> PermissionFlags {
    PermissionFlags(flags)
  }

  pub fn empty() -> PermissionFlags {
    PermissionFlags(0)
  }

  pub fn as_u8(&self) -> u8 {
    self.0
  }
}

pub trait PageDirectory {
  fn map(&self, frame: Frame, vaddr: VirtualAddress, flags: PermissionFlags);
}

pub struct CurrentPageDirectory {
}

impl CurrentPageDirectory {
  pub fn get() -> CurrentPageDirectory {
    CurrentPageDirectory {}
  }

  pub fn unmap(&self, vaddr: VirtualAddress) {
    let dir_index = vaddr.get_page_directory_index();
    let table_index = vaddr.get_page_table_index();
    let directory = PageTable::at_address(VirtualAddress::new(0xfffff000));
    if !directory.get(dir_index).is_present() {
      return;
    }
    let table = PageTable::at_address(VirtualAddress::new(
      0xffc00000 + 0x1000 * dir_index,
    ));
    if !table.get(table_index).is_present() {
      return;
    }
    table.get_mut(table_index).clear_present();
    invalidate_page(vaddr);
  }

  pub fn unmap_region(&self, region: VirtualMemoryRegion) {
    let mut page_start = VirtualAddress::new(region.get_starting_address_as_usize());
    while region.contains_address(page_start) {
      self.unmap(page_start);
      page_start = page_start.offset(0x1000);
    }
  }
}

impl PageDirectory for CurrentPageDirectory {
  fn map(&self, frame: Frame, vaddr: VirtualAddress, flags: PermissionFlags) {
    let paddr = frame.get_address();
    let dir_index = vaddr.get_page_directory_index();
    let table_index = vaddr.get_page_table_index();
    let top_page = PageTable::at_address(VirtualAddress::new(0xfffff000));
    // Address for the nested page table
    let table_address = VirtualAddress::new(0xffc00000 + (dir_index * 0x1000));

    let entry = top_page.get_mut(dir_index);
    if !entry.is_present() {
      // Create a page table
      let table_frame = allocate_frame().unwrap();
      entry.set_address(table_frame.get_address());
      entry.set_present();
      if dir_index < 768 {
        entry.set_user_access();
        entry.set_write_access();
      }
      let table = PageTable::at_address(table_address);
      table.zero();
      table.get_mut(table_index).set_address(paddr);
      table.get_mut(table_index).set_present();
      if flags.as_u8() & PermissionFlags::WRITE_ACCESS != 0 {
        table.get_mut(table_index).set_write_access();
      }
      if flags.as_u8() & PermissionFlags::USER_ACCESS != 0 {
        table.get_mut(table_index).set_user_access();
      }
    } else {
      let table = PageTable::at_address(table_address);
      let needs_invalidation = table.get(table_index).is_present();
      table.get_mut(table_index).set_address(paddr);
      table.get_mut(table_index).set_present();
      if flags.as_u8() & PermissionFlags::WRITE_ACCESS != 0 {
        table.get_mut(table_index).set_write_access();
      }
      if flags.as_u8() & PermissionFlags::USER_ACCESS != 0 {
        table.get_mut(table_index).set_user_access();
      }
      if needs_invalidation {
        invalidate_page(vaddr);
      }
    }
  }
}

pub struct AlternatePageDirectory {
  directory_address: PhysicalAddress,
}

impl AlternatePageDirectory {
  pub fn new(addr: PhysicalAddress) -> AlternatePageDirectory {
    AlternatePageDirectory {
      directory_address: addr,
    }
  }

  pub fn map_region(&self, region: VirtualMemoryRegion) {
    match region.backing_type() {
      MemoryRegionType::Direct(_) | MemoryRegionType::IO(_) => {
        // Copy the mappings directly
        panic!("Direct/IO mapping not implemented");
      },
      MemoryRegionType::MemMapped(_, _, _) => {
        match region.get_permissions() {
          Permissions::ReadOnly => {
            // Copy the mappings directly
          },
          Permissions::ReadWrite => {
            // Copy data to entirely new frames
          },
          Permissions::CopyOnWrite => {
            // Copy mapping with write permmission disabled
          },
        }
        panic!("MemMapping not implemented");
      },
      MemoryRegionType::Anonymous(_) => {
        match region.get_permissions() {
          Permissions::ReadOnly => {
            // Copy the mappings directly
            self.copy_mapping_directly(region);
          },
          Permissions::ReadWrite => {
            // Copy data to entirely new frames
            self.copy_frames(region);
          },
          Permissions::CopyOnWrite => {
            // Copy mapping with write permission disabled
            self.map_with_copy_on_write(region);
          },
        }
      },
    }
  }

  fn copy_mapping_directly(&self, region: VirtualMemoryRegion) {
    let mut page_start = VirtualAddress::new(region.get_starting_address_as_usize());
    while region.contains_address(page_start) {
      let directory_index = page_start.get_page_directory_index();
      let directory = PageTable::at_address(VirtualAddress::new(0xfffff000));
      if directory.get(directory_index).is_present() {
        let table_index = page_start.get_page_table_index();
        let table_address = VirtualAddress::new(0xffc00000 + 0x1000 * directory_index);
        let table = PageTable::at_address(table_address);
        if table.get(table_index).is_present() {
          let frame_paddr = table.get(table_index).get_address();
          let mut flags = 0;
          if table.get(table_index).is_user_access_granted() {
            flags |= PermissionFlags::USER_ACCESS;
          }
          if table.get(table_index).is_write_access_granted() {
            flags |= PermissionFlags::WRITE_ACCESS;
          }

          self.map(
            Frame::new(frame_paddr.as_usize()),
            page_start,
            PermissionFlags::new(flags),
          );
        }
      }
      page_start = page_start.offset(0x1000);
    }
  }

  fn copy_frames(&self, region: VirtualMemoryRegion) {
    let mut page_start = VirtualAddress::new(region.get_starting_address_as_usize());
    while region.contains_address(page_start) {
      let directory_index = page_start.get_page_directory_index();
      let directory = PageTable::at_address(VirtualAddress::new(0xfffff000));
      if directory.get(directory_index).is_present() {
        let table_index = page_start.get_page_table_index();
        let table_address = VirtualAddress::new(0xffc00000 + 0x1000 * directory_index);
        let table = PageTable::at_address(table_address);
        if table.get(table_index).is_present() {
          let new_frame = allocate_frame().unwrap();
          map_frame_to_temporary_page(new_frame);

          unsafe {
            let mut from_ptr = page_start.as_usize() as *const u32;
            let mut to_ptr = get_temporary_page_address().as_usize() as *mut u32;
            for i in 0..1024 {
              *to_ptr = *from_ptr;
              from_ptr = from_ptr.offset(1);
              to_ptr = to_ptr.offset(1);
            }
          }

          let mut flags = PermissionFlags::WRITE_ACCESS;
          if page_start.as_usize() < 0xc0000000 {
            flags |= PermissionFlags::USER_ACCESS;
          }

          self.map(
            new_frame,
            page_start,
            PermissionFlags::new(flags),
          );
        }
      }
      page_start = page_start.offset(0x1000);
    }
  }

  fn map_with_copy_on_write(&self, region: VirtualMemoryRegion) {
    let mut page_start = VirtualAddress::new(region.get_starting_address_as_usize());
    while region.contains_address(page_start) {
      let directory_index = page_start.get_page_directory_index();
      let directory = PageTable::at_address(VirtualAddress::new(0xfffff000));
      if directory.get(directory_index).is_present() {
        let table_index = page_start.get_page_table_index();
        let table_address = VirtualAddress::new(0xffc00000 + 0x1000 * directory_index);
        let table = PageTable::at_address(table_address);
        if table.get(table_index).is_present() {
          let frame_paddr = table.get(table_index).get_address();
          reference_frame_at_address(frame_paddr);
          let flags = if table.get(table_index).is_user_access_granted() {
            PermissionFlags::USER_ACCESS
          } else {
            0
          };
          // force no write access and revoke current write permissions,
          // so that the first write duplicates the frame
          table.get_mut(table_index).clear_write_access();

          self.map(
            Frame::new(frame_paddr.as_usize()),
            page_start,
            PermissionFlags::new(flags),
          );
        }
      }
      page_start = page_start.offset(0x1000);
    }
  }
}

impl PageDirectory for AlternatePageDirectory {
  fn map(&self, frame: Frame, vaddr: VirtualAddress, flags: PermissionFlags) {
    let pagedir_frame = Frame::new(self.directory_address.as_usize());
    map_frame_to_temporary_page(pagedir_frame);
    let dir_index = vaddr.get_page_directory_index();
    let table_index = vaddr.get_page_table_index();
    let directory = PageTable::at_address(get_temporary_page_address());
    if !directory.get(dir_index).is_present() {
      // Allocate a page table
      let table_frame = allocate_frame().unwrap();
      directory.get_mut(dir_index).set_address(table_frame.get_address());
      directory.get_mut(dir_index).set_present();
      if dir_index < 768 {
        directory.get_mut(dir_index).set_user_access();
        directory.get_mut(dir_index).set_write_access();
      }
      map_frame_to_temporary_page(table_frame);
      let table = PageTable::at_address(get_temporary_page_address());
      table.zero();
      table.get_mut(table_index).set_address(frame.get_address());
      table.get_mut(table_index).set_present();
      if flags.as_u8() & PermissionFlags::WRITE_ACCESS != 0 {
        table.get_mut(table_index).set_write_access();
      }
      if flags.as_u8() & PermissionFlags::USER_ACCESS != 0 {
        table.get_mut(table_index).set_user_access();
      }
    } else {
      let addr = directory.get(dir_index).get_address();
      map_frame_to_temporary_page(Frame::new(addr.as_usize()));
      let table = PageTable::at_address(get_temporary_page_address());
      let needs_invalidation = table.get(table_index).is_present();
      table.get_mut(table_index).set_address(frame.get_address());
      table.get_mut(table_index).set_present();
      if flags.as_u8() & PermissionFlags::WRITE_ACCESS != 0 {
        table.get_mut(table_index).set_write_access();
      }
      if flags.as_u8() & PermissionFlags::USER_ACCESS != 0 {
        table.get_mut(table_index).set_user_access();
      }
      if needs_invalidation {
        invalidate_page(vaddr);
      }
    }
  }
}

#[cfg(not(test))]
pub fn set_current_pagedir(addr: PhysicalAddress) {
  crate::x86::registers::set_cr3(addr.as_u32());
}

#[cfg(not(test))]
pub fn get_current_pagedir() -> PhysicalAddress {
  let cr3 = crate::x86::registers::get_cr3();
  PhysicalAddress::new(cr3 as usize)
}

#[cfg(not(test))]
pub fn invalidate_page(addr: VirtualAddress) {
  unsafe {
    llvm_asm!("invlpg ($0)" : : "r"(addr.as_u32()) : "memory");
  }
}

pub fn get_temporary_page_address() -> VirtualAddress {
  VirtualAddress::new(0xffbff000)
}

pub fn get_current_page_address() -> VirtualAddress {
  VirtualAddress::new(0xfffff000)
}

pub fn map_frame_to_temporary_page(frame: Frame) {
  // The temporary page is located in the last slot of the second-to-last page
  // table. Assuming the current pagedir is mapped to its own last slot, this
  // means the entry we want to edit is the one just prior to the last 4KiB of
  // virtual memory.
  let last_table = PageTable::at_address(VirtualAddress::new(0xffffe000));
  last_table.get_mut(1023).set_address(frame.get_address());
  last_table.get_mut(1023).set_present();
  invalidate_page(get_temporary_page_address());
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

#[cfg(test)]
pub fn invalidate_page(_addr: VirtualAddress) {
  // no-op in tests
}
