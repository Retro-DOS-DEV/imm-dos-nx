/**
 * Split a path into its drive and local path components
 */
pub fn string_to_drive_and_path(raw: &str) -> (&str, &str) {
  let mut drive_split = raw.splitn(2, ':');
  let drive = match drive_split.next() {
    Some(d) => d,
    None => {
      return (&raw[0..0], &raw[0..0]);
    },
  };
  let path = drive_split.next();
  match path {
    None => {
      // There was no colon present in the path
      // Treat this situation as having no drive
      (&raw[0..0], drive)
    },
    Some(p) => {
      (drive, p)
    },
  }
}
