use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::ops::DerefMut;
use crate::memory::virt::map_kernel_stack;
use spin::RwLock;
use super::id::{IDGenerator, ProcessID};
use super::process::Process;

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

}

pub fn initialize() {
  let idle_task = super::process::Process::initial(0);
  let id = *idle_task.get_id();
  let entry = Arc::new(RwLock::new(idle_task));
  let mut map = TASK_MAP.write();
  map.insert(id, entry);
}

pub fn get_process(id: &ProcessID) -> Option<Arc<RwLock<Process>>> {
  let map = TASK_MAP.read();
  let entry = map.get(id)?;
  Some(entry.clone())
}

pub fn get_current_process() -> Arc<RwLock<Process>> {
  let current_id: ProcessID = *CURRENT_ID.read();
  let map = TASK_MAP.read();
  let entry = map.get(&current_id).expect("Current process does not exist!");
  entry.clone()
}

/// When a process gets forked, we create a duplicate process with an empty
/// stack. Previously the kernel used a bunch of hacks to duplicate the stack
/// and ensure that the child process returned through all the callers in the
/// same way the parent did. However, all we really need is for the child to
/// return to the userspace entrypoint with the same registers.
/// When a process enters a syscall, we store a pointer to the
pub fn fork(current_ticks: u32) -> ProcessID {
  let current_process = get_current_process();
  let next_id = NEXT_ID.next();
  let child = {
    let parent = current_process.read();
    parent.create_fork(next_id, current_ticks)
  };
  map_kernel_stack(child.get_stack_range());
  {
    let mut map = TASK_MAP.write();
    map.insert(next_id, Arc::new(RwLock::new(child)));
  }
  next_id
}

pub fn kfork(dest: extern "C" fn() -> ()) -> ProcessID {
  let child_id = fork(0);
  {
    let child_lock = get_process(&child_id).unwrap();
    let mut child = child_lock.write();
    child.stack_push_u32(dest as u32);
    crate::kprintln!("Child %esp: {:#0x}", child.stack_pointer);
  }
  crate::kprintln!("Child will start at {:#0x}", dest as u32);
  child_id
}

/// Execute a context switch to another process. If that process does not exist,
/// the method will panic.
pub fn switch_to(id: &ProcessID) {
  let mut current_ptr = None;
  let mut next_ptr = None;
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
  crate::kprintln!("JUMP TO {:?}", *id);
  unsafe {
    let current = &mut *current_ptr.unwrap();
    let next = &mut *next_ptr.unwrap();
    llvm_asm!("push eax; push ecx; push edx; push ebx; push ebp; push esi; push edi" : : : "esp" : "intel", "volatile");
    switch_inner(current, next);
    llvm_asm!("pop edi; pop esi; pop ebp; pop ebx; pop edx; pop ecx; pop eax" : : : "esp" : "intel", "volatile");
  }
}

#[naked]
#[inline(never)]
unsafe fn switch_inner(current: &mut Process, next: &mut Process) {
  asm!(
    "mov {0}, esp
    mov esp, {1}",
    out(reg) (current.stack_pointer),
    in(reg) (next.stack_pointer),
  );
}

/// Jump to a process and force it to enter usermode.
pub fn usermode_enter(id: &ProcessID) {
  let mut current_ptr = None;
  let mut next_ptr = None;
  {
    let current_lock = get_current_process();
    let mut current = current_lock.write();
    current_ptr = Some(current.deref_mut() as *mut Process);
    let next_lock = get_process(id).unwrap();
    let mut next = next_lock.write();
    next_ptr = Some(next.deref_mut() as *mut Process);
  }
  *CURRENT_ID.write() = *id;
  unsafe {
    let current = &mut *current_ptr.unwrap();
    let next = &mut *next_ptr.unwrap();
    llvm_asm!("push eax; push ecx; push edx; push ebx; push ebp; push esi; push edi" : : : "esp" : "intel", "volatile");
    usermode_enter_inner(current, next);
    llvm_asm!("pop edi; pop esi; pop ebp; pop ebx; pop edx; pop ecx; pop eax" : : : "esp" : "intel", "volatile");
  }
}

#[naked]
#[inline(never)]
unsafe fn usermode_enter_inner(current: &mut Process, next: &mut Process) {
  asm!(
    "mov {0}, esp
    mov esp, {1}
    iretd",
    out(reg) (current.stack_pointer),
    in(reg) (next.stack_pointer),
  );
}