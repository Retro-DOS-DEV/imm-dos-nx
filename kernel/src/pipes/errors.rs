#[derive(Debug)]
pub enum PipeError {
  /// The specified handle does not point to a valid pipe
  InvalidHandle,
  /// A pipe handle pointed to an unknown pipe, should not happen
  UnknownPipe,
  /// Attempted to use a read handle to write, or vice-versa
  WrongHandleType,
  /// Writing to a pipe with no readers
  WriteToClosedPipe,
}