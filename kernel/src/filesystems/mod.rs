use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;

pub mod dev;
pub mod filesystem;

pub type DriveName = [u8; 8];

pub type FileSystemType = dyn filesystem::FileSystem + Send + Sync;

pub struct FileSystemMap {
  map: BTreeMap<DriveName, Arc<Box<FileSystemType>>>,
}

impl FileSystemMap {

}

// Temporary
pub static DEV: dev::DevFileSystem = dev::DevFileSystem::new();
