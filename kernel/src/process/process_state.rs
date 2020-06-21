use crate::files::handle::{DeviceHandlePair, FileHandle, FileHandleMap, LocalHandle};
use crate::memory;
use crate::memory::address::VirtualAddress;
use crate::memory::virt::page_directory;
use crate::memory::virt::page_table::{PageTable, PageTableReference};
use crate::memory::virt::region::{MemoryRegionType, VirtualMemoryRegion};
use spin::RwLock;
use super::id::ProcessID;

pub struct ProcessState {
  pid: ProcessID,
  parent: ProcessID,
  kernel_heap_region: RwLock<VirtualMemoryRegion>,
  kernel_stack_region: RwLock<VirtualMemoryRegion>,
  page_directory: PageTableReference,

  open_files: RwLock<FileHandleMap>,
}

impl ProcessState {
  /**
   * Used to generate the init process, which has no parent
   */
  pub fn first(pid: ProcessID, heap_start: VirtualAddress) -> ProcessState {
    ProcessState {
      pid,
      parent: pid,
      kernel_heap_region: RwLock::new(
        VirtualMemoryRegion::new(
          heap_start,
          memory::heap::INITIAL_HEAP_SIZE * 0x1000,
          MemoryRegionType::Anonymous,
        ),
      ),
      kernel_stack_region: RwLock::new(
        VirtualMemoryRegion::new(
          memory::virt::STACK_START,
          0x1000,
          MemoryRegionType::Anonymous,
        ),
      ),
      page_directory: PageTableReference::current(),

      open_files: RwLock::new(FileHandleMap::new()),
    }
  }

  /**
   * It is not possible to create an orphaned process; each process must be
   * forked from an existing one.
   */
  pub fn fork(&self, pid: ProcessID) -> ProcessState {
    ProcessState {
      pid,
      parent: self.pid,
      kernel_heap_region: RwLock::new(
        self.kernel_heap_region.read().clone(),
      ),
      kernel_stack_region: RwLock::new(
        VirtualMemoryRegion::new(
          memory::virt::STACK_START,
          0x1000,
          MemoryRegionType::Anonymous,
        ),
      ),
      page_directory: self.fork_page_directory(),

      open_files: RwLock::new(FileHandleMap::new()),
    }
  }

  pub fn fork_page_directory(&self) -> PageTableReference {
    let temp_page_address = page_directory::get_temporary_page_address();
    // Create the top page table, which will contain the temp page and
    // kernel stack
    let top_page = memory::physical::allocate_frame().unwrap();
    page_directory::map_frame_to_temporary_page(top_page);
    PageTable::at_address(temp_page_address).zero();
    // Create the new page directory
    let pagedir_frame = memory::physical::allocate_frame().unwrap();
    page_directory::map_frame_to_temporary_page(pagedir_frame);
    let new_pagedir = PageTable::at_address(temp_page_address);
    new_pagedir.zero();
    // Initialize the page directory with its required mappings
    new_pagedir.get_mut(1023).set_address(pagedir_frame.get_address());
    new_pagedir.get_mut(1023).set_present();
    new_pagedir.get_mut(1022).set_address(top_page.get_address());
    new_pagedir.get_mut(1022).set_present();

    // Copy the kernel heap mappings
    let current_page_address = page_directory::get_current_page_address();
    let current_pagedir = PageTable::at_address(current_page_address);
    {
      let heap_region = self.kernel_heap_region.read();
      let start = heap_region.get_starting_address_as_usize();
      let size = heap_region.get_size();
      let mut offset = 0;
      while offset < size {
        let index = (start + offset) >> 22;
        new_pagedir.get_mut(index).set_address(current_pagedir.get(index).get_address());
        new_pagedir.get_mut(index).set_present();
        offset += 0x400000;
      }
    }

    // No need to create a stack mapping, since we can now map on demand

    // Duplicate the process memory mapping
    // Right now this just copies the kernel data. This needs to come from a
    // process-stored map in the future.
    new_pagedir.get_mut(0).set_address(current_pagedir.get(0).get_address());
    new_pagedir.get_mut(0).set_present();
    new_pagedir.get_mut(0x300).set_address(current_pagedir.get(0x300).get_address());
    new_pagedir.get_mut(0x300).set_present();

    PageTableReference::new(pagedir_frame.get_address())
  }

  pub fn get_page_directory(&self) -> &PageTableReference {
    &self.page_directory
  }

  pub fn get_kernel_heap_region(&self) -> &RwLock<VirtualMemoryRegion> {
    &self.kernel_heap_region
  }

  pub fn get_kernel_stack_region(&self) -> &RwLock<VirtualMemoryRegion> {
    &self.kernel_stack_region
  }

  pub fn open_file(&self, drive: usize, local: LocalHandle) -> FileHandle {
    let mut files = self.open_files.write();
    files.open_handle(drive, local)
  }

  pub fn close_file(&self, handle: FileHandle) {
    let mut files = self.open_files.write();
    files.close_handle(handle)
  }

  pub fn get_open_file_info(&self, handle: FileHandle) -> Option<DeviceHandlePair> {
    let files = self.open_files.read();
    files.get_drive_and_handle(handle)
  }
}