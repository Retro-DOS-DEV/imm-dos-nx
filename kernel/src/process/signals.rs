use super::all_processes;
use super::id::ProcessID;
use super::process_state::{BlockReason, ProcessState, RunState};

fn exit_code(signal: u32, code: u32) -> u32 {
  ((code & 0xff) << 8) | (signal & 0x7f)
}

impl ProcessState {
  /// Handle a signal number
  pub fn send_signal(&self, sig: u32) {
    match sig {
      syscall::signals::STOP | syscall::signals::TSTOP => {
        let mut run_state = self.get_run_state().write();
        *run_state = RunState::Paused;
      },
      syscall::signals::CONTINUE => {
        let mut run_state = self.get_run_state().write();
        if *run_state == RunState::Paused {
          *run_state = RunState::Running;
        }
      },

      syscall::signals::INT |
      syscall::signals::TERM => {
        // TODO: Check if there is a signal handler

        self.terminate(sig, 0);
      },
      syscall::signals::KILL => {
        self.terminate(sig, 0);
      },

      _ => (),
    }
  }

  /// Put the process in uninterruptible sleep, waiting for a resource. It can
  /// only be awoken with the resume() call.
  pub fn block(&self) {
    let mut run_state = self.get_run_state().write();
    *run_state = RunState::Blocked(BlockReason::None);
  }

  pub fn block_on_child(&self, id: ProcessID) {
    let mut run_state = self.get_run_state().write();
    *run_state = RunState::Blocked(BlockReason::Child(id));
  }

  pub fn resume(&self) {
    let mut run_state = self.get_run_state().write();
    match *run_state {
      RunState::Blocked(_) => {
        *run_state = RunState::Running;
      },
      _ => (),
    }
  }

  /// Kill the process, either because the process called exit() or a
  /// terminating signal was sent
  pub fn terminate(&self, signal: u32, code: u32) {
    self.set_exit_code(exit_code(signal, code));
    let mut run_state = self.get_run_state().write();
    *run_state = RunState::Terminated;

    // Tell the parent process that the child has terminated
    let current_id = self.get_id();
    let parent_id = self.get_parent();
    let processes = all_processes();
    match processes.get_process(parent_id) {
      Some(parent) => parent.child_exited(current_id, code),
      None => (),
    }
  }

  pub fn exit(&self, code: u32) {
    self.terminate(0, code);
  }

  pub fn child_exited(&self, child: ProcessID, code: u32) {
    self.send_signal(syscall::signals::CHILD);
    let mut run_state = self.get_run_state().write();
    match *run_state {
      RunState::Blocked(BlockReason::Child(id)) => {
        if id == child {
          *run_state = RunState::Resumed(code);
        }
      },
      _ => (),
    }
  }

  pub fn get_resume_code(&self) -> u32 {
    let mut run_state = self.get_run_state().write();
    match *run_state {
      RunState::Resumed(code) => {
        *run_state = RunState::Running;
        code
      },
      _ => 0,
    }
  }
}