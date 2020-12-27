use super::id::ProcessID;

pub struct Process {
  id: ProcessID,

  parent_id: ProcessID,
}
