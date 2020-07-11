use crate::files::handle::{DriveHandlePair, FileHandle, FileHandleMap, LocalHandle};
use crate::memory;
use crate::memory::address::VirtualAddress;
use crate::memory::physical::frame::Frame;
use crate::memory::virt::page_directory;
use crate::memory::virt::page_table::{PageTable, PageTableReference};
use crate::memory::virt::region::{ExpansionDirection, MemoryRegionType, Permissions, VirtualMemoryRegion};
use crate::time;
use spin::RwLock;
use super::id::ProcessID;
use super::memory::MemoryRegions;
use super::subsystem::Subsystem;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RunState {
  Running,
  Sleeping(usize),
  Paused,
  // Blocked(PromiseResult),
}

pub struct ProcessState {
  pid: ProcessID,
  parent: ProcessID,

  memory_regions: RwLock<MemoryRegions>,

  page_directory: PageTableReference,

  kernel_esp: RwLock<usize>,

  open_files: RwLock<FileHandleMap>,

  run_state: RwLock<RunState>,
  subsystem: RwLock<Subsystem>,
}

impl ProcessState {
  /**
   * Used to generate the init process, which has no parent
   */
  pub fn first(pid: ProcessID, heap_start: VirtualAddress) -> ProcessState {
    ProcessState {
      pid,
      parent: pid,

      memory_regions: RwLock::new(MemoryRegions::initial(heap_start)),

      page_directory: PageTableReference::current(),

      kernel_esp: RwLock::new(0),

      open_files: RwLock::new(FileHandleMap::new()),

      run_state: RwLock::new(RunState::Running),
      subsystem: RwLock::new(Subsystem::Native),
    }
  }

  /**
   * It is not possible to create an orphaned process; each process must be
   * forked from an existing one.
   */
  pub fn fork(&self, pid: ProcessID) -> ProcessState {
    let new_regions = RwLock::new(self.memory_regions.read().fork());
    let new_pagedir = self.fork_page_directory();
    let new_filemap = self.fork_file_map();
    ProcessState {
      pid,
      parent: self.pid,

      memory_regions: new_regions,

      page_directory: new_pagedir,

      kernel_esp: RwLock::new(
        memory::virt::STACK_START.as_usize() + 0x1000 - 4
      ),

      open_files: RwLock::new(new_filemap),

      run_state: RwLock::new(RunState::Running),
      subsystem: RwLock::new(Subsystem::Native),
    }
  }

  pub fn make_current_stack_frame_editable(&self) {
    let esp = self.kernel_esp.read().clone();
    let directory_entry = esp >> 22;
    let table_entry = (esp >> 12) & 0x3ff;
    // Map the page table into temp space
    page_directory::map_frame_to_temporary_page(Frame::new(self.page_directory.get_address().as_usize()));
    let temp_page_address = page_directory::get_temporary_page_address();
    let pagedir = PageTable::at_address(temp_page_address);
    let stack_table_address = pagedir.get(directory_entry).get_address().as_usize();
    page_directory::map_frame_to_temporary_page(Frame::new(stack_table_address));
    if !pagedir.get(table_entry).is_present() {
      let stack_frame = memory::physical::allocate_frame().unwrap();
      pagedir.get_mut(table_entry).set_address(stack_frame.get_address());
      pagedir.get_mut(table_entry).set_present();
    }
    let current_stack_frame = pagedir.get(table_entry).get_address().as_usize();
    page_directory::map_frame_to_temporary_page(Frame::new(current_stack_frame));
  }

  pub fn set_initial_entry_point(&self, func: extern fn(), esp: usize) {
    self.make_current_stack_frame_editable();
    let temp_page_address = page_directory::get_temporary_page_address().as_usize();
    let kernel_esp = self.kernel_esp.read().clone();
    let stack_offset = kernel_esp & 0xfff;
    unsafe {
      let stack_ptr = (temp_page_address + stack_offset) as *mut usize;
      // Stack segment
      *stack_ptr.offset(-1) = 0x23;
      // Stack pointer
      *stack_ptr.offset(-2) = esp;
      // eflags
      *stack_ptr.offset(-3) = 0x200; // interrupt enabled
      // Code segment
      *stack_ptr.offset(-4) = 0x1b;
      // Instruction pointer
      *stack_ptr.offset(-5) = (func as usize) & 0x3fffffff; 
    }
    *self.kernel_esp.write() = kernel_esp - 4 * 5;
  }

  pub fn set_kernel_mode_entry_point(&self, func: extern fn()) {
    let stack_addr = 0xffbfeff8;
    *self.kernel_esp.write() = stack_addr;
    self.make_current_stack_frame_editable();
    let temp_page_address = page_directory::get_temporary_page_address().as_usize();
    unsafe {
      let stack_ptr = 0xffbffff8 as *mut usize;
      *stack_ptr = func as usize; 
    }
  }

  pub fn get_range_containing_address(&self, addr: VirtualAddress) -> Option<VirtualMemoryRegion> {
    self.memory_regions.read().get_range_containing_address(addr)
  }

  pub fn get_id(&self) -> ProcessID {
    self.pid
  }

  pub fn get_page_directory(&self) -> &PageTableReference {
    &self.page_directory
  }

  pub fn get_memory_regions(&self) -> &RwLock<MemoryRegions> {
    &self.memory_regions
  }

  pub fn get_kernel_stack_pointer(&self) -> usize {
    self.kernel_esp.read().clone()
  }

  pub fn get_kernel_stack_container(&self) -> &RwLock<usize> {
    &self.kernel_esp
  }

  pub fn get_open_files(&self) -> &RwLock<FileHandleMap> {
    &self.open_files
  }

  pub fn get_subsystem(&self) -> &RwLock<Subsystem> {
    &self.subsystem
  }

  pub fn sleep(&self, ms: usize) {
    let mut run_state = self.run_state.write();
    *run_state = RunState::Sleeping(ms);
  }

  pub fn update_tick(&self) {
    let run_state = self.run_state.read().clone();
    match run_state {
      RunState::Sleeping(duration) => {
        if duration > time::system::MS_PER_TICK {
          let remaining = duration - time::system::MS_PER_TICK;
          *self.run_state.write() = RunState::Sleeping(remaining);
          return;
        }
        *self.run_state.write() = RunState::Running;
      },
      _ => (),
    }
  }

  pub fn is_running(&self) -> bool {
    let run_state = self.run_state.read().clone();
    match run_state {
      RunState::Running => true,
      _ => false
    }
  }

  pub fn get_run_state(&self) -> &RwLock<RunState> {
    &self.run_state
  }
}
