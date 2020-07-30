pub enum SeekMethod {
  Absolute(usize),
  Relative(isize),
}

impl SeekMethod {
  pub fn from_current_position(&self, current: usize) -> usize {
    match self {
      SeekMethod::Absolute(pos) => *pos,
      SeekMethod::Relative(off) => (current as isize).saturating_add(*off) as usize,
    }
  }
}
