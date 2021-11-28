use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::ops::DerefMut;
use crate::memory::physical::{reference_frame_at_address};
use crate::memory::address::VirtualAddress;
use crate::memory::virt::map_kernel_stack;
use crate::memory::virt::page_table::PageTableReference;
use spin::RwLock;
use super::id::{IDGenerator, ProcessID};
use super::paging;
use super::process::Process;
use super::stack::UnmappedPage;

/// The task map allows fetching process information by ID. It's also used for
/// scheduling, to determine which process should run next.
/// Previous versions of the kernel used locks for every mutable field in the
/// process, rather than placing the whole process in a single lock. This
/// created a lot of extra code and room for potential deadlocks, though, so
/// the map has been simplified.
pub static TASK_MAP: RwLock<BTreeMap<ProcessID, Arc<RwLock<Process>>>> = RwLock::new(BTreeMap::new());

/// Used to generate incrementing process IDs
pub static NEXT_ID: IDGenerator = IDGenerator::new();

/// All kernel code referencing the "current" process will use this ID
pub static CURRENT_ID: RwLock<ProcessID> = RwLock::new(ProcessID::new(0));

/// Cooperatively yield, forcing the scheduler to switch to another process
pub fn yield_coop() {
  let next = find_next_running_process();
  match next {
    Some(id) => switch_to(&id),
    None => (),
  }
}

pub fn initialize() {
  let idle_task = super::process::Process::initial(0);
  let id = *idle_task.get_id();
  let entry = Arc::new(RwLock::new(idle_task));
  let mut map = TASK_MAP.write();
  map.insert(id, entry);
}

/// Find another process to switch to. If non is available (eg, we are currently
/// in the idle task and all other tasks are blocked), it will return None.
/// For now, our switching algo is simple: find the first process whose ID comes
/// after the current ID. If none is found, return the first runnable process we
/// encountered in our first pass.
pub fn find_next_running_process() -> Option<ProcessID> {
  let current_id = *CURRENT_ID.read();
  let mut first_runnable = None;
  let task_map = TASK_MAP.read();
  for (id, process) in task_map.iter() {
    if *id == current_id {
      continue;
    }
    if process.read().can_resume() {
      if first_runnable.is_none() {
        first_runnable.replace(*id);
      }
      if *id > current_id {
        return Some(*id);
      }
    }
  }
  // If we hit the end of the list, loop back to the first running process
  // we found. If there is none, we stay on the current process.
  first_runnable
}

pub fn get_process(id: &ProcessID) -> Option<Arc<RwLock<Process>>> {
  let map = TASK_MAP.read();
  let entry = map.get(id)?;
  Some(entry.clone())
}

pub fn get_current_id() -> ProcessID {
  *CURRENT_ID.read()
}

pub fn get_current_process() -> Arc<RwLock<Process>> {
  let current_id: ProcessID = *CURRENT_ID.read();
  let map = TASK_MAP.read();
  let entry = map.get(&current_id).expect("Current process does not exist!");
  entry.clone()
}

pub fn for_each_process<F>(f: F)
  where F: Fn(Arc<RwLock<Process>>) -> () {
  for (_, proc) in TASK_MAP.read().iter() {
    f(proc.clone());
  }
}

pub fn for_each_process_mut<F>(mut f: F)
  where F: FnMut(Arc<RwLock<Process>>) -> () {
  for (_, proc) in TASK_MAP.read().iter() {
    f(proc.clone());
  }
}

/// When a process gets forked, we create a duplicate process with an empty
/// stack. Previously the kernel used a bunch of hacks to duplicate the stack
/// and ensure that the child process returned through all the callers in the
/// same way the parent did. However, all we really need is for the child to
/// return to the userspace entrypoint with the same registers.
/// When a process enters a syscall, we store a pointer to the
pub fn fork(current_ticks: u32, include_userspace: bool) -> ProcessID {
  let current_process = get_current_process();
  let next_id = NEXT_ID.next();
  let mut child = {
    let parent = current_process.read();
    parent.create_fork(next_id, current_ticks)
  };
  super::io::reopen_files(*child.get_id(), &mut child.open_files);
  {
    // re-open the executable file
    match super::io::reopen_executable(*child.get_id(), child.get_exec_file()) {
      Some((drive, handle)) => {
        child.set_exec_file(drive, handle);
      },
      None => {
        child.remove_exec_file();
      },
    }
  }
  map_kernel_stack(child.get_stack_range());
  child.page_directory = fork_page_directory(include_userspace);
  super::stack::duplicate_stack(
    current_process.read().get_kernel_stack(),
    child.get_kernel_stack_mut(),
  );
  // Move the stack pointer down past the 5 values from the interrupt,
  // and the 10 values pushed by the syscall wrapper.
  // It should return within the syscall wrapper, popping off the registers and
  // returning to userspace.
  child.stack_pointer -= 5 * core::mem::size_of::<u32>();
  child.stack_push_u32(0); // replace eax with 0 in the child
  child.stack_pointer -= 9 * core::mem::size_of::<u32>();
  //crate::kprintln!("Child {:?} ({:?}) stack: {:?}", next_id, current_process.read().get_id(), child.get_stack_range());
  {
    let mut map = TASK_MAP.write();
    map.insert(next_id, Arc::new(RwLock::new(child)));
  }
  next_id
}

pub fn kfork(dest: extern "C" fn() -> ()) -> ProcessID {
  let child_id = fork(0, false);
  {
    let child_lock = get_process(&child_id).unwrap();
    let mut child = child_lock.write();
    child.stack_push_u32(dest as u32);
    //crate::kprintln!("Child %esp: {:#0x}", child.stack_pointer);
  }
  //crate::kprintln!("Child will start at {:#0x}", dest as u32);
  child_id
}

pub fn clean_up_process(id: ProcessID) {
  let task = {
    let mut task_map = TASK_MAP.write();
    match task_map.remove(&id) {
      Some(t) => t,
      None => return,
    }
  };
  crate::kprintln!("Clean up {:?}", task.read().get_id());
}

/// Execute a context switch to another process. If that process does not exist,
/// the method will panic.
pub fn switch_to(id: &ProcessID) {
  let current_ptr;
  let next_ptr;
  {
    // Nasty deref_mut hacks to get around the locks
    let current_lock = get_current_process();
    let mut current = current_lock.write();
    current_ptr = Some(current.deref_mut() as *mut Process);
    let next_lock = get_process(id).unwrap();
    let mut next = next_lock.write();
    next_ptr = Some(next.deref_mut() as *mut Process);
  }
  *CURRENT_ID.write() = *id;
  //crate::kprintln!("JUMP TO {:?}", *id);
  unsafe {
    let current = &mut *current_ptr.unwrap();
    let next = &mut *next_ptr.unwrap();
    crate::gdt::set_tss_stack_pointer(next.get_stack_range().end.as_u32() - 4);
    llvm_asm!("push eax; push ecx; push edx; push ebx; push ebp; push esi; push edi" : : : "esp" : "intel", "volatile");
    {
      let pagedir_addr = next.page_directory.get_address().as_usize();
      let current_sp_addr = &mut current.stack_pointer as *mut usize as usize;
      let next_sp = next.stack_pointer;
      switch_inner(pagedir_addr, current_sp_addr, next_sp);
    }
    llvm_asm!("pop edi; pop esi; pop ebp; pop ebx; pop edx; pop ecx; pop eax" : : : "esp" : "intel", "volatile");
  }
}

#[naked]
#[inline(never)]
unsafe extern "cdecl" fn switch_inner(_pagedir_addr: usize, _current_sp_addr: usize, _next_sp: usize) {
  asm!(
    "mov eax, [esp + {arg_size}]
    mov ecx, [esp + 2 * {arg_size}]
    mov edx, [esp + 3 * {arg_size}]
    mov cr3, eax
    mov [ecx], esp
    mov esp, edx
    ret",
    arg_size = const core::mem::size_of::<usize>(),
    options(noreturn),
  );
}

pub fn update_timeouts(delta_ms: usize) {
  let task_map = TASK_MAP.read();
  for (_, process) in task_map.iter() {
    process.write().update_timeouts(delta_ms);
  }
}

pub fn fork_page_directory(include_userspace: bool) -> PageTableReference {
  use crate::memory::physical;
  use crate::memory::virt::page_table;

  // Create a new page directory
  let directory_frame = physical::allocate_frame().unwrap().to_frame();
  let directory_scratch_space = UnmappedPage::map(directory_frame.get_address());
  let directory_table = page_table::PageTable::at_address(directory_scratch_space.virtual_address());
  directory_table.zero();
  // Set the top entry to itself
  directory_table.get_mut(1023).set_address(directory_frame.get_address());
  directory_table.get_mut(1023).set_present();
  // Copy all other kernel-space entries from the current table
  let current_directory = page_table::PageTable::at_address(VirtualAddress::new(0xfffff000));
  for entry in 0x300..0x3ff {
    if current_directory.get(entry).is_present() {
      let table_address = current_directory.get(entry).get_address();
      directory_table.get_mut(entry).set_address(table_address);
      directory_table.get_mut(entry).set_present();
    }
  }

  if include_userspace {
    // Copy the user-space entries from the current table, making any writable
    // table entries copy-on-write
    for dir_entry in 0..0x300 {
      if !current_directory.get(dir_entry).is_present() {
        continue;
      }
      let table_address = VirtualAddress::new(0xffc00000 + (dir_entry * 0x1000));
      let table = page_table::PageTable::at_address(table_address);
      for table_index in 0..1024 {
        let table_entry = table.get_mut(table_index);
        if table_entry.is_present() {
          // All entries, writable or not, now have an additional reference
          let _ = reference_frame_at_address(table_entry.get_address())
            // since the entire page table is being copied, there is no call to
            // .map and we can safely dispose of this AllocatedFrame
            .to_frame();

          if table_entry.is_write_access_granted() {
            table_entry.clear_write_access();
            table_entry.set_cow();
            {
              let page_start =
                (dir_entry * 4 * 1024 * 1024)
                + table_index * 4 * 1024;
              paging::invalidate_page(VirtualAddress::new(page_start));
            }
            crate::kprintln!("SET COW {} {}", dir_entry, table_index);
          }

          let ref_count = crate::memory::physical::get_current_refcount_for_address(table_entry.get_address());
          crate::kprintln!("{:?} count is now {}", table_entry.get_address(), ref_count);
        }
      }
      let table_frame = paging::duplicate_frame(table_address).to_frame();
      directory_table.get_mut(dir_entry).set_address(table_frame.get_address());
      directory_table.get_mut(dir_entry).set_user_access();
      directory_table.get_mut(dir_entry).set_present();
      if current_directory.get(dir_entry).is_write_access_granted() {
        directory_table.get_mut(dir_entry).set_write_access();
      }
    }
  }

  PageTableReference::new(directory_frame.get_address())
}
