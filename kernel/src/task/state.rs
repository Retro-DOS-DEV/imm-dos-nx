use super::id::ProcessID;

/// RunState represents the current state of the process, and determines how the
/// kernel handles the process. It is mostly used to represent ways that an
/// existing process may not be actively running.
/// 
/// When a process is Running, the kernel assumes it can be safely executed. The
/// scheduler will consider this process when switching tasks.
/// 
/// When a process crashes, exits, or is killed by a signal, it moves to a
/// Terminated state containing an optional exit code. This allows the process
/// struct to remain in memory without executing, so that the kernel can notify
/// its parent process and clean up the resources associated with the terminated
/// process. A kernel-level process regularly walks the process map and handles
/// these terminated programs.
/// 
/// Sleeping is used when a process wants to pause execution and yield the CPU
/// to other processes for a fixed period of time. When a process enters sleep,
/// it specifies how long it should sleep for. The Sleeping state stores a
/// counter of time remaining before the process should resume. At regular
/// intervals, the kernel updates the counter for all sleeping proceses. When
/// this counter reaches zero, the process state is replaced with Running.
/// 
///             Call Sleep(n)
///   [Running] ------------> [Sleeping(n)] --
///       ^                      |   ^        | Kernel updates the remaining
///       |                      |   |        | value on an interrupt
///       | n has reached zero   |    --------
///        ----------------------
/// 
/// A process can be Paused by external signals. It needs to be woken up by a
/// different signal to return to Running state.
/// 
/// When a process chooses to listen for IPC messages, it switches to
/// AwaitingIPC state. The scheduler will not enter this process, but IPC
/// messages sent to it will cause the kernel to synchronously jump directly to
/// the listening process.
/// 
/// A process can block on a child until it exits. While blocked, its state is
/// set to WaitingForChild. When that child exits, its return code is sent to
/// the blocked process. The parent process sets its state to Resumed, storing
/// the return code. The next time the scheduler enters that process, it sets up
/// the registers to return that code, updates to a Running state, and resumes
/// execution.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RunState {
  /// Running normally
  Running,
  /// Process has exited, or been terminated. The kernel should clean it up.
  Terminated,
  /// Sleeping for a fixed amount of time
  Sleeping(usize),
  /// Paused because of a signal
  Paused,
  /// Waiting for IPC messages
  AwaitingIPC,
  /// Waiting for a child process to finish executing
  WaitingForChild(ProcessID),
  /// Just resumed from a waiting state. This is quickly replaced with a Running
  /// state once the return code has been processed.
  Resumed(u32),
  /// Currently executing a signal handler. If a memory violation occurs, the
  /// process's stack and registers can be restored, after which it enters a
  /// Running state.
  HandlingSignal(u32),
  /// Similar to handling a signal, allows user-mode handling of interrupts
  HandlingInterrupt(u32),
}
