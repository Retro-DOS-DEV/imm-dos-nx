use alloc::string::String;

/// Path represents a full absolute path within a drive. It should neither start
/// nor end with directory separators
pub struct Path {
  raw: String,
}

impl Path {
  pub fn new(raw: &str) -> Path {
    Path { raw: String::from(raw.trim_end_matches('\\').trim_start_matches('\\')) }
  }

  pub fn as_str(&self) -> &str {
    self.raw.as_str()
  }

  /// Construct a path by applying a local path to a current-working-dir path
  pub fn resolve(cwd: &str, local: &str) -> Path {
    if local.starts_with('\\') {
      // absolute path
      return Path::new(local.trim_start_matches('\\'));
    }
    let mut path = Path::new(cwd);
    let sections = local.split('\\');
    for section in sections {
      path.add(section);
    }

    path
  }

  fn remove_last(&mut self) {
    let mut last_instance = None;
    for (index, ch) in self.raw.char_indices() {
      if ch == '\\' {
        last_instance = Some(index);
      }
    }
    match last_instance {
      Some(index) => self.raw.truncate(index),
      None => self.raw.truncate(0),
    }
  }

  pub fn add(&mut self, sub: &str) {
    match sub {
      "." => (), // same dir, do nothing
      ".." => { // parent directory
        self.remove_last();
      },
      _ => {
        if !self.raw.is_empty() {
          self.raw.push('\\');
        }
        self.raw.push_str(sub);
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Path;

  #[test]
  fn construction() {
    assert_eq!(Path::new("abc\\d\\efghi").as_str(), "abc\\d\\efghi");
    assert_eq!(Path::new("\\absolute\\path").as_str(), "absolute\\path");
    assert_eq!(Path::new("some\\nested\\dirs\\").as_str(), "some\\nested\\dirs");
  }

  #[test]
  fn add_subdir() {
    let mut path = Path::new("chain\\of");
    path.add("dirs");
    assert_eq!(path.as_str(), "chain\\of\\dirs");
    path.add(".");
    assert_eq!(path.as_str(), "chain\\of\\dirs");
    path.add("..");
    assert_eq!(path.as_str(), "chain\\of");
  }

  #[test]
  fn joining_paths() {
    assert_eq!(
      Path::resolve("current\\working\\dir", "..\\subdir\\test.txt").as_str(),
      "current\\working\\subdir\\test.txt",
    );
    assert_eq!(
      Path::resolve("aaa\\bbb\\ccc", "..\\..\\..\\..\\..\\..\\..\\foo.bar").as_str(),
      "foo.bar",
    );
  }
}
