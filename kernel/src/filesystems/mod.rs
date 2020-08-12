use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

#[cfg(not(test))]
pub mod dev;
#[cfg(not(test))]
pub mod init;

pub mod fat12;
pub mod filesystem;

pub type FileSystemType = dyn filesystem::FileSystem + Send + Sync;

pub struct FileSystemNumber(usize);

impl FileSystemNumber {
  pub fn new(index: usize) -> FileSystemNumber {
    FileSystemNumber(index)
  }

  pub fn as_usize(&self) -> usize {
    self.0
  }
}

pub struct NamedFileSystem(pub Box<str>, pub Arc<Box<FileSystemType>>);

impl NamedFileSystem {
  pub fn matches_name(&self, name: &str) -> bool {
    self.0.as_ref() == name
  }

  pub fn get_fs(&self) -> Arc<Box<FileSystemType>> {
    self.1.clone()
  }
}

pub struct FileSystemMap {
  map: RwLock<Vec<NamedFileSystem>>,
}

impl FileSystemMap {
  pub const fn new() -> FileSystemMap {
    FileSystemMap {
      map: RwLock::new(Vec::new()),
    }
  }

  pub fn register_fs(&self, name: &str, fs: Box<FileSystemType>) -> Result<usize, ()> {
    let mut map = self.map.write();
    map.push(NamedFileSystem(Box::from(name), Arc::new(fs)));
    Ok(map.len() - 1)
  }

  pub fn get_fs_number(&self, name: &str) -> Option<usize> {
    let map = self.map.read();
    let mut index = 0;
    for entry in map.iter() {
      if entry.matches_name(name) {
        return Some(index);
      }
      index += 1;
    }
    None
  }

  pub fn get_fs(&self, index: usize) -> Option<Arc<Box<FileSystemType>>> {
    let map = self.map.read();
    let entry = map.get(index)?;
    Some(entry.get_fs())
  }
}

pub static VFS: FileSystemMap = FileSystemMap::new();

pub static mut DEV_FS: usize = 0;
pub static mut PIPE_FS: usize = 0;

pub fn get_fs_number(name: &str) -> Option<usize> {
  VFS.get_fs_number(name)
}

pub fn get_fs(index: usize) -> Option<Arc<Box<FileSystemType>>> {
  VFS.get_fs(index)
}

#[cfg(not(test))]
pub fn init_fs() {
  let dev_fs = dev::DevFileSystem::new();
  let dev_number = VFS.register_fs("DEV", Box::new(dev_fs)).expect("Failed to register DEV FS");
  let pipe_fs = crate::pipes::create_fs();
  let pipe_number = VFS.register_fs("PIPE", pipe_fs).expect("Failed to register PIPE FS");
  unsafe {
    PIPE_FS = pipe_number;
    DEV_FS = dev_number;
  }
}
