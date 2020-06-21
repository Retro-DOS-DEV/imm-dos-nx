use crate::files::handle::{DeviceHandlePair, FileHandle, FileHandleMap, LocalHandle};
use crate::memory;
use crate::memory::address::VirtualAddress;
use crate::memory::virt::region::{MemoryRegionType, VirtualMemoryRegion};
use spin::RwLock;
use super::id::ProcessID;

pub struct ProcessState {
  pid: ProcessID,
  parent: ProcessID,
  kernel_heap_region: RwLock<VirtualMemoryRegion>,
  kernel_stack_region: RwLock<VirtualMemoryRegion>,

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

      open_files: RwLock::new(FileHandleMap::new()),
    }
  }

  /**
   * It is not possible to create an orphaned process; each process must be
   * forked from an existing one.
   */
  pub fn fork(&self, pid: ProcessID, parent: ProcessID) -> ProcessState {
    ProcessState {
      pid,
      parent,
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

      open_files: RwLock::new(FileHandleMap::new()),
    }
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