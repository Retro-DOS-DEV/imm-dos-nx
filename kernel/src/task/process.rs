use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::files::handle::{FileHandle, Handle, LocalHandle};
use crate::fs::drive::DriveID;
use crate::memory::address::VirtualAddress;
use crate::memory::virt::page_table::PageTableReference;
use super::files::{FileMap, OpenFile};
use super::id::ProcessID;
use super::ipc::{IPCMessage, IPCPacket, IPCQueue};
use super::memory::{ExecutionSegment, MemoryRegions, Relocation};
use super::regs::SavedState;
use super::state::RunState;
use super::vm::Subsystem;

pub const MAX_PROCESS_COUNT: usize = 256 * 64 - 1;

pub struct Process {
  /// The unique ID of this process
  id: ProcessID,
  /// The ID of the parent process
  parent_id: ProcessID,
  /// Stores the details of all addresses mapped into the process's memory.
  /// When a page fault occurs, this information is used to determine how
  /// content is paged into memory, or if it's a crash-causing fault.
  pub memory: MemoryRegions,
  /// Represents the current execution state of the process
  state: RunState,
  /// The number of system ticks when this process was started
  start_ticks: u32,
  /// Stores IPC messages that have been sent to this process
  ipc_queue: IPCQueue,
  /// Stores references to all currently open files
  pub open_files: FileMap,
  /// A Box pointing to the kernel stack for this process. Each stack is page-
  /// aligned, and exists in a reserved area of kernel memory space. The lowest
  /// page of each stack is read/write protected, to act as a guard page.
  /// The stack Box is wrapped in an Option so that we can replace it with None
  /// before the struct is dropped. If any code attempts to drop the stack Box,
  /// it will cause an error because the memory wasn't allocated on the heap.
  pub kernel_stack: Option<Box<[u8]>>,
  /// Stores the kernel stack pointer when the process is swapped out. When the
  /// scheduler enters this process, this address will be placed in %esp and all
  /// registers will be popped off the stack.
  pub stack_pointer: usize,
  /// Store the register state when the process is interrupted
  pub saved_state: SavedState,
  /// A struct containing the physical address of this process's page directory.
  /// When switching to this process, the address will be written to CR3.
  pub page_directory: PageTableReference,
  /// Reference to the open file being executed by this process
  exec_file: Option<(DriveID, LocalHandle)>,
  /// Stores the relocation data necessary for setting up the executable file in
  /// memory.
  relocations: Vec<Relocation>,
  /// Stores extra data related to the subsystem used by the process
  pub subsystem: Subsystem,
  /// An optional kernel-level method to run when exiting VM86 mode
  pub on_exit_vm: Option<usize>,
  /// If set, points to the VTerm that initialized this process or its ancestor
  vterm: Option<usize>,
}

impl Process {
  /// Generate the init process. This shouldn't be called more than once.
  pub fn initial(current_ticks: u32) -> Self {
    super::stack::allocate_initial_stacks();
    let kernel_stack = super::stack::stack_box_from_index(1);

    Self {
      id: ProcessID::new(0),
      parent_id: ProcessID::new(0),
      memory: MemoryRegions::new(),
      state: RunState::Running,
      start_ticks: current_ticks,
      ipc_queue: IPCQueue::new(),
      open_files: FileMap::with_capacity(3),
      kernel_stack: Some(kernel_stack),
      stack_pointer: 0,
      saved_state: SavedState::empty(),
      page_directory: PageTableReference::current(),
      exec_file: None,
      relocations: Vec::new(),
      subsystem: Subsystem::Native,
      on_exit_vm: None,
      vterm: None,
    }
  }

  pub fn get_id(&self) -> &ProcessID {
    &self.id
  }

  pub fn get_parent_id(&self) -> &ProcessID {
    &self.parent_id
  }

  pub fn get_exec_file(&self) -> Option<(DriveID, LocalHandle)> {
    self.exec_file
  }

  /// Based on the current system time in ticks, how long has this process been
  /// running?
  pub fn uptime_ticks(&self, current_ticks: u32) -> u32 {
    current_ticks - self.start_ticks
  }

  /// Determine if the scheduler can re-enter this process
  pub fn can_resume(&self) -> bool {
    match self.state {
      RunState::Running | RunState::Resumed(_) => true,
      _ => false,
    }
  }

  /// Get a reference to the kernel stack
  pub fn get_kernel_stack(&self) -> &Box<[u8]> {
    match &self.kernel_stack {
      Some(stack) => stack,
      None => panic!("Process does not have a stack"),
    }
  }

  pub fn get_kernel_stack_mut(&mut self) -> &mut Box<[u8]> {
    match &mut self.kernel_stack {
      Some(stack) => stack,
      None => panic!("Process does not have a stack"),
    }
  }

  pub fn reset_stack_pointer(&mut self) {
    let stack_end = self.get_stack_range().end;
    self.stack_pointer = stack_end.as_usize() - 4;
  }

  /// Return the virtual address range for this process's stack
  pub fn get_stack_range(&self) -> core::ops::Range<VirtualAddress> {
    let (stack_start, len) = {
      let stack = self.get_kernel_stack();
      (stack.as_ptr() as usize, stack.len())
    };
    let start = VirtualAddress::new(stack_start);
    let end = start + len;
    start..end
  }

  pub fn stack_push_u8(&mut self, value: u8) {
    self.stack_pointer -= 1;
    let esp = self.stack_pointer;
    let stack = self.get_kernel_stack_mut();
    let start = stack.as_ptr() as usize;
    let offset = esp - start;
    stack[offset] = value;
  }

  pub fn stack_pop_u8(&mut self) -> u8 {
    let value = {
      let esp = self.stack_pointer;
      let stack = self.get_kernel_stack_mut();
      let start = stack.as_ptr() as usize;
      let offset = esp - start;
      stack[offset]
    };
    self.stack_pointer += 1;
    value
  }

  pub fn stack_push_u32(&mut self, value: u32) {
    self.stack_pointer -= 4;
    let esp = self.stack_pointer;
    let stack = self.get_kernel_stack_mut();
    let start = stack.as_ptr() as usize;
    let offset = esp - start;
    stack[offset] = (value & 0xff) as u8;
    stack[offset + 1] = ((value & 0xff00) >> 8) as u8;
    stack[offset + 2] = ((value & 0xff0000) >> 16) as u8;
    stack[offset + 3] = ((value & 0xff000000) >> 24) as u8;
  }

  /// Used to force a kernel process into usermode. This should only be used
  /// for testing, and not in the real kernel.
  pub fn set_usermode_entrypoint(&mut self, func: extern fn(), esp: u32) {
    // Stack segment
    self.stack_push_u32(0x23);
    // Stack pointer
    self.stack_push_u32(esp);
    // eflags
    self.stack_push_u32(0x200); // Interrupts enabled
    // Code segment
    self.stack_push_u32(0x1b);
    // Instruction pointer
    self.stack_push_u32(func as u32 & 0x3fffffff);
  }

  /// Force a process to think it was started by the specified vterm. This is
  /// used for the initial process in each vterm.
  pub fn force_vterm(&mut self, index: usize) {
    self.vterm = Some(index);
  }

  pub fn get_vterm(&self) -> Option<usize> {
    self.vterm
  }

  /// End all execution of the process, and mark its resources for cleanup.
  pub fn terminate(&mut self) {
    self.state = RunState::Terminated;
  }

  /// Pause this process for a specified number of milliseconds. When the
  /// duration has passed, the process's state will return to Running.
  pub fn sleep(&mut self, duration: usize) {
    self.state = RunState::Sleeping(duration);
  }

  /// Pause the process due to a signal. It will not resume until woken by
  /// a different signal.
  pub fn pause(&mut self) {
    self.state = RunState::Paused;
  }

  /// Resume the process due to a signal. If the process is not explicitly
  /// paused, this is a no-op.
  pub fn resume(&mut self) {
    match self.state {
      RunState::Paused => self.state = RunState::Running,
      _ => (),
    }
  }

  pub fn wait(&mut self, child_id: Option<ProcessID>) {
    self.state = RunState::WaitingForChild(child_id);
  }

  pub fn resume_from_wait(&mut self) -> u32 {
    match self.state {
      RunState::Resumed(code) => {
        self.state = RunState::Running;
        return code;
      },
      _ => 0,
    }
  }

  /// Tell a process that a child has exited. If the process is currently
  /// waiting on that child, it will resume execution.
  pub fn child_returned(&mut self, child_id: ProcessID, code: u32) {
    let waiting_on = match self.state {
      RunState::WaitingForChild(id) => id,
      _ => return,
    };
    match waiting_on {
      None => self.state = RunState::Resumed(code),
      Some(id) if id == child_id => self.state = RunState::Resumed(code),
      _ => (),
    }
  }

  /// Attempt to read an IPC message. If none is available, the process will
  /// block until a message is received or the optional timeout argument
  /// expires. When the process unblocks, it should re-issue a call to this
  /// method.
  /// Because entries in the IPC queue are only expired when it is read or
  /// written, the current time needs to be passed to this method to clean up
  /// any items that are due for removal.
  pub fn ipc_read(&mut self, current_ticks: u32, timeout: Option<usize>) -> (Option<IPCPacket>, bool) {
    let (first_read, has_more) = self.ipc_queue.read(current_ticks);
    if first_read.is_some() {
      return (first_read, has_more);
    }
    // Nothing in the queue, block the process until something arrives
    self.state = RunState::AwaitingIPC(timeout);
    (None, false)
  }

  /// Unblocking version of ipc_read
  pub fn ipc_read_unblocking(&mut self, current_ticks: u32) -> (Option<IPCPacket>, bool) {
    self.ipc_queue.read(current_ticks)
  }

  /// Send an IPC message to this process. If the process is currently blocked
  /// on reading the IPC queue, it will wake up.
  /// Each message is accompanied by an expiration time (in system ticks), after
  /// which point the message will be considered invalid if it hasn't been read.
  pub fn ipc_receive(&mut self, current_ticks: u32, from: ProcessID, message: IPCMessage, expiration_ticks: u32) {
    self.ipc_queue.add(from, message, current_ticks, expiration_ticks);
    match self.state {
      RunState::AwaitingIPC(_) => {
        self.state = RunState::Running;
      },
      _ => (),
    }
  }

  /// Update any internal timers based on regular system clock updates.
  pub fn update_timeouts(&mut self, delta_ms: usize) {
    match self.state {
      RunState::AwaitingIPC(Some(timeout)) => {
        self.state = if timeout < delta_ms {
          RunState::Running
        } else {
          RunState::AwaitingIPC(Some(timeout - delta_ms))
        };
      },
      RunState::Sleeping(timeout) => {
        self.state = if timeout < delta_ms {
          RunState::Running
        } else {
          RunState::Sleeping(timeout - delta_ms)
        };
      },
      _ => (),
    }
  }

  /// Increase the process heap by a specific number of bytes. The old heap
  /// endpoint will be returned.
  pub fn increase_heap(&mut self, increment: usize) -> VirtualAddress {
    let start = self.memory.get_heap_start();
    let prev_size = self.memory.get_heap_size();
    self.memory.set_heap_size(prev_size + increment);
    start + prev_size
  }

  /// Prepare for an exec syscall by removing the current execution segments and
  /// mmap mappings, and replacing them with a new set of segments.
  pub fn prepare_exec_mapping(&mut self, exec: Vec<ExecutionSegment>) -> Vec<ExecutionSegment> {
    let previous_exec = self.memory.reset_execution_segments(exec);
    // TODO: remove all mmap entries
    previous_exec
  }

  /// Change the reference to the executable file being run in this process.
  /// When a page fault occurs within an executable section, the fault handler
  /// will use this to look up the file and fill the missing page.
  pub fn set_exec_file(&mut self, drive_id: DriveID, handle: LocalHandle) -> Option<(DriveID, LocalHandle)> {
    self.exec_file.replace((drive_id, handle))
  }

  pub fn remove_exec_file(&mut self) -> Option<(DriveID, LocalHandle)> {
    self.exec_file.take()
  }

  pub fn set_relocations(&mut self, relocations: Vec<Relocation>) {
    self.relocations = relocations;
  }

  pub fn get_relocations(&self) -> &Vec<Relocation> {
    &self.relocations
  }

  /// Create a copy of this process and its memory space.
  pub fn create_fork(&self, new_id: ProcessID, current_ticks: u32) -> Process {
    let new_stack = super::stack::allocate_stack();
    let stack_top = (new_stack.as_ptr() as usize) + new_stack.len() - 4;

    Process {
      id: new_id,
      parent_id: self.id,
      memory: self.memory.clone(),
      state: RunState::Running,
      start_ticks: current_ticks,
      ipc_queue: IPCQueue::new(),
      open_files: self.open_files.clone(),
      kernel_stack: Some(new_stack),
      stack_pointer: stack_top,
      saved_state: SavedState::empty(),
      page_directory: self.page_directory.clone(),
      exec_file: self.exec_file,
      relocations: self.relocations.clone(),
      subsystem: Subsystem::Native,
      on_exit_vm: None,
      vterm: self.vterm,
    }
  }

  /// When a file has been opened within a specific drive, it can be added to
  /// this process. The index in the file map is returned as a FileHandle.
  pub fn open_file(&mut self, drive: DriveID, local_handle: LocalHandle) -> FileHandle {
    let file = OpenFile {
      drive,
      local_handle,
    };
    let index = self.open_files.insert(file);
    FileHandle::new(index as u32)
  }

  pub fn get_open_file_info(&self, handle: FileHandle) -> Option<&OpenFile> {
    self.open_files.get(handle.as_usize())
  }

  /// Close an open file handle. If it represented a file within a drive, a
  /// struct containing that drive's ID and its local handle will be returned.
  pub fn close_file(&mut self, handle: FileHandle) -> Option<OpenFile> {
    self.open_files.remove(handle.as_usize())
  }

  /// Duplicate an existing descriptor, possibly to a specific handle. It
  /// returns the previous open file descriptor if one was overwritten, and the
  /// file handle that was created.
  pub fn duplicate_file_descriptor(&mut self, old: FileHandle, new: Option<FileHandle>) -> (Option<OpenFile>, Option<FileHandle>) {
    let copied_entry = match self.open_files.get(old.as_usize()) {
      Some(entry) => *entry,
      None => return (None, None),
    };
    match new {
      Some(new_handle) => {
        let old_value = self.open_files.replace(new_handle.as_usize(), copied_entry);
        (old_value, Some(new_handle))
      },
      None => {
        let new_handle = FileHandle::new(self.open_files.insert(copied_entry) as u32);
        (None, Some(new_handle))
      },
    }
  }

  /// Mark a process as blocked on file IO
  pub fn io_block(&mut self, timeout: Option<usize>) {
    self.state = RunState::FileIO(timeout);
  }

  /// If a process is blocked on file IO, wake it up
  pub fn io_resume(&mut self) {
    match self.state {
      RunState::FileIO(_) => {
        self.state = RunState::Running;
      },
      _ => (),
    }
  }

  /// Mark a process as blocked on hardware IO
  pub fn hardware_block(&mut self, timeout: Option<usize>) {
    self.state = RunState::HardwareIO(timeout);
  }

  /// If a process is blocked on hardware IO, wake it up
  pub fn hardware_resume(&mut self) {
    match self.state {
      RunState::HardwareIO(_) => {
        self.state = RunState::Running;
      },
      _ => (),
    }
  }

  /// Save a set of stashed registers from a memory location
  pub fn save_state(&mut self, state: &SavedState) {
    self.saved_state = *state;
  }

  /// Restore the last saved set of registers to a memory location
  pub fn restore_state(&self, state: &mut SavedState) {
    *state = self.saved_state;
  }
}

impl Drop for Process {
  fn drop(&mut self) {
    // Make sure it doesn't attempt to deallocate the stack Box
    let stack = self.kernel_stack.take();
    if let Some(b) = stack {
      super::stack::free_stack(b);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{DriveID, FileHandle, Handle, LocalHandle, Process, VirtualAddress};

  #[test]
  fn sleeping() {
    let mut p = Process::initial(0);
    p.sleep(2000);
    assert!(!p.can_resume());
    p.update_timeouts(500);
    p.update_timeouts(1000);
    assert!(!p.can_resume());
    p.update_timeouts(700);
    assert!(p.can_resume());
  }

  #[test]
  fn heap_modification() {
    let mut p = Process::initial(0);
    // just put the heap endpoint somewhere
    p.increase_heap(0x250);
    // simulate `brk`
    {
      let prev = p.increase_heap(0);
      p.increase_heap(VirtualAddress::new(0x1200) - prev);
      assert_eq!(p.memory.get_heap_start() + p.memory.get_heap_size(), VirtualAddress::new(0x1200));
    }
    // simulate `sbrk`
    {
      let prev = p.increase_heap(0);
      assert_eq!(prev, p.increase_heap(0x430));
      assert_eq!(prev + 0x430, p.memory.get_heap_start() + p.memory.get_heap_size());
    }
  }

  #[test]
  fn file_handle_dup() {
    let mut p = Process::initial(0);
    let first = p.open_file(DriveID::new(0), LocalHandle::new(2));
    let second = p.open_file(DriveID::new(1), LocalHandle::new(0));
    let third = p.open_file(DriveID::new(0), LocalHandle::new(4));
    {
      // `dup` syscall
      let (prev_entry, new_handle) = p.duplicate_file_descriptor(second, None);
      assert!(prev_entry.is_none());
      assert_eq!(new_handle, Some(FileHandle::new(3)));
    }

    {
      // `dup2` to a new location
      let (prev_entry, new_handle) = p.duplicate_file_descriptor(first, Some(FileHandle::new(6)));
      assert!(prev_entry.is_none());
      assert_eq!(new_handle, Some(FileHandle::new(6)));
    }

    {
      // `dup2` override
      let (prev_entry, new_handle) = p.duplicate_file_descriptor(third, Some(FileHandle::new(0)));
      let open_file = prev_entry.unwrap();
      assert_eq!(open_file.drive, DriveID::new(0));
      assert_eq!(open_file.local_handle, LocalHandle::new(2));
      assert_eq!(new_handle, Some(FileHandle::new(0)));
    }
  }
}
