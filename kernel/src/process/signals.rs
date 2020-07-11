use super::process_state::{ProcessState, RunState};

impl ProcessState {
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

      _ => (),
    }
  }
}