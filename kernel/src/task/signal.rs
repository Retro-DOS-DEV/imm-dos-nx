/// Subset of POSIX signals, useful for modifying process state
pub enum Signal {
  Segfault,
  UserInterrupt,
  UserQuit,
}
