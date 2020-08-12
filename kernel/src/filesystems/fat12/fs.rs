use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::devices;
use crate::drivers::driver::DeviceDriver;
use crate::files::cursor::SeekMethod;
use crate::files::handle::{Handle, HandleAllocator, LocalHandle};
use crate::memory::address::VirtualAddress;
use spin::RwLock;
use super::directory::{Directory, DirectoryEntry, DirectoryEntryIterator};
use super::disk::{BiosParamBlock, DiskConfig, DIRECTORY_ENTRY_SIZE};
use super::fat::{Cluster, ClusterChain, FatEntry, FatSection, FatValueResult};
use super::file::{FileType, file_name_components_from_string};
use super::super::filesystem::FileSystem;
use syscall::files::{DirEntryInfo, DirEntryType};

struct OpenFile {
  pub cursor: usize,
  pub file_type: FileType,
  pub clusters: ClusterChain,
}

pub struct Fat12FileSystem {
  handle_allocator: HandleAllocator<LocalHandle>,
  open_files: RwLock<BTreeMap<LocalHandle, OpenFile>>,

  drive_number: usize,
  drive_access_handle: LocalHandle,

  config: DiskConfig,
  io_buffer: RwLock<Vec<u8>>,
}

impl Fat12FileSystem {
  pub fn new(drive_number: usize, drive_access_handle: LocalHandle) -> Fat12FileSystem {
    let mut io_buffer = Vec::with_capacity(512);
    for _ in 0..512 {
      io_buffer.push(0);
    }
    Fat12FileSystem {
      handle_allocator: HandleAllocator::new(),
      open_files: RwLock::new(BTreeMap::new()),

      drive_number,
      drive_access_handle,

      config: DiskConfig::empty(),
      io_buffer: RwLock::new(io_buffer),
    }
  }

  pub fn init(&mut self) -> Result<(), ()> {
    let driver = devices::get_driver_for_device(self.drive_number).ok_or(())?;
    driver.open(self.drive_access_handle)?;
    driver.seek(self.drive_access_handle, SeekMethod::Absolute(0x0b))?;
    let mut bpb = BiosParamBlock::empty();
    driver.read(self.drive_access_handle, bpb.as_buffer())?;
    self.config.from_bpb(&bpb);
    Ok(())
  }

  fn get_io_buffer_address(&self) -> VirtualAddress {
    VirtualAddress::new(self.io_buffer.read().as_ptr() as usize)
  }

  fn get_fat_sector_for_cluster(&self, cluster: Cluster) -> usize {
    let clusters_per_sector = self.config.get_bytes_per_sector() * 2 / 3 + 1;
    cluster.as_usize() / clusters_per_sector
  }

  fn load_sector_of_fat_table(&self, table: usize, sector: usize) -> Result<(), ()> {
    if sector >= self.config.get_sectors_per_fat() {
      return Err(())
    }

    let fat_sectors = self.config.get_fat_sectors(table).map_err(|_| ())?;
    let sector_index = fat_sectors.get_first_sector() + sector;
    let position = self.config.get_bytes_per_sector() * sector_index;

    let driver = devices::get_driver_for_device(self.drive_number).ok_or(())?;
    driver.seek(self.drive_access_handle, SeekMethod::Absolute(position))?;
    {
      let mut buffer = self.io_buffer.write();
      driver.read(self.drive_access_handle, buffer.as_mut_slice())?;
    }
    Ok(())
  }

  pub fn get_cluster_chain(&self, first_cluster: Cluster) -> Result<ClusterChain, ()> {
    let mut clusters = Vec::with_capacity(1);
    let mut next = FatEntry::NextCluster(first_cluster);
    let mut current_fat_sector = 0xffff;
    let mut fat_sector_byte_offset = 0;
    let mut first_cluster_in_fat_sector = Cluster::new(0);

    let clusters_per_sector = self.config.get_bytes_per_sector() * 2 / 3 + 1;

    while let FatEntry::NextCluster(c) = next {
      clusters.push(c);

      let sector = self.get_fat_sector_for_cluster(c);
      if sector != current_fat_sector {
        self.load_sector_of_fat_table(0, sector);

        first_cluster_in_fat_sector = Cluster::new(clusters_per_sector * sector);
        
        if sector > 0 {
          let prev_trailing_bytes = sector * self.config.get_bytes_per_sector() % 3;
          fat_sector_byte_offset = 3 - prev_trailing_bytes;
        } else {
          fat_sector_byte_offset = 0;
        }

        current_fat_sector = sector;
      }
      
      let value = {
        let mut buffer = self.io_buffer.write();
        FatSection::at_slice(buffer.as_mut_slice(), fat_sector_byte_offset, first_cluster_in_fat_sector)
          .get_value(c)
      };
      match value {
        FatValueResult::Partial4(part) => {
          self.load_sector_of_fat_table(0, sector + 1);
          current_fat_sector += 1;
          first_cluster_in_fat_sector = Cluster::new(
            first_cluster_in_fat_sector.as_usize() + clusters_per_sector
          );
          fat_sector_byte_offset = 1;

          let high = self.io_buffer.read()[0] as u16;
          next = FatEntry::from_value((part as u16) | (high << 4));
        },
        FatValueResult::Partial8(part) => {
          self.load_sector_of_fat_table(0, sector + 1);
          current_fat_sector += 1;
          first_cluster_in_fat_sector = Cluster::new(
            first_cluster_in_fat_sector.as_usize() + clusters_per_sector
          );
          fat_sector_byte_offset = 2;

          let high = (self.io_buffer.read()[0] & 0x0f) as u16;
          next = FatEntry::from_value((part as u16) | (high << 8));
        },
        FatValueResult::Success(entry) => {
          next = entry;
        },
        _ => (),
      }
    }

    Ok(ClusterChain::from_vec(clusters))
  }

  pub fn find_entry_in_directory(&self, name: &[u8; 8], ext: &[u8; 3], search_dir: Directory) -> Result<DirectoryEntry, ()> {
    let driver = devices::get_driver_for_device(self.drive_number).ok_or(())?;
    for sector in search_dir.clusters.sector_iter(&self.config) {
      let bytes_per_sector = self.config.get_bytes_per_sector();
      let position = sector * bytes_per_sector;
      driver.seek(self.drive_access_handle, SeekMethod::Absolute(position))?;
      {
        let mut buffer = self.io_buffer.write();
        driver.read(self.drive_access_handle, buffer.as_mut_slice())?;
      }

      let entries_per_sector = bytes_per_sector / DIRECTORY_ENTRY_SIZE;
      let buffer_addr = self.get_io_buffer_address();
      let mut entry_count = 0;
      for entry in DirectoryEntryIterator::new(buffer_addr, entries_per_sector) {
        entry_count += 1;
        if entry.name_matches_search(&name, &ext) {
          return Ok(*entry);
        }
      }
      if entry_count < entries_per_sector {
        return Err(());
      }
    }
    Err(())
  }
}

impl FileSystem for Fat12FileSystem {
  fn open(&self, path: &str) -> Result<LocalHandle, ()> {
    
    // First, find the file
    // Iterate through each directory name in the path, until coming to the
    // parent directory
    let mut parts = path.split("\\");
    let mut search_dir = Directory::empty();

    let mut part = parts.next().ok_or(())?;
    loop {
      // find the directory with the matching name, and fetch its cluster chain

      // to be implemented...

      let next = parts.next();
      match next {
        Some(p) => part = p,
        None => break,
      }
    }

    // With the parent directory located, iterate through all directory entries
    // to find a file with a matching name
    let (name, ext) = file_name_components_from_string(part);

    let entry = self.find_entry_in_directory(&name, &ext, search_dir)?;
    let first_cluster = entry.get_first_cluster();
    let cluster_chain = self.get_cluster_chain(first_cluster)?;
    let open_file = OpenFile {
      cursor: 0,
      file_type: FileType::File,
      clusters: cluster_chain,
    };
    let handle = self.handle_allocator.get_next();
    self.open_files.write().insert(handle, open_file);

    crate::kprintln!("FOUND FILE {:?}.{:?}", name, ext);
    Ok(handle)
  }

  fn read(&self, handle: LocalHandle, buffer: &mut [u8]) -> Result<usize, ()> {
    Err(())
  }

  fn write(&self, handle: LocalHandle, buffer: &[u8]) -> Result<usize, ()> {
    Err(())
  }

  fn close(&self, handle: LocalHandle) -> Result<(), ()> {
    Err(())
  }

  fn dup(&self, handle: LocalHandle) -> Result<LocalHandle, ()> {
    Err(())
  }

  fn seek(&self, handle: LocalHandle, offset: SeekMethod) -> Result<usize, ()> {
    Err(())
  }

  fn open_dir(&self, path: &str) -> Result<LocalHandle, ()> {
    let handle = self.handle_allocator.get_next();

    let dir = Directory::empty(); // Root directory
    let open_file = OpenFile {
      cursor: 0,
      file_type: FileType::Directory,
      clusters: dir.clusters,
    };
    self.open_files.write().insert(handle, open_file);
    Ok(handle)
  }

  fn read_dir(&self, handle: LocalHandle, index: usize, info: &mut DirEntryInfo) -> Result<(), ()> {
    let (sector, local_index) = {
      let files = self.open_files.read();
      let file = files.get(&handle).ok_or(())?;
      let (dir_sector, local_index) = self.config.get_directory_index_location(index);
      let mut iter = file.clusters.sector_iter(&self.config);
      for _ in 0..dir_sector {
        iter.next();
      }
      let sector = iter.next().ok_or(())?;

      (sector, local_index)
    };

    let position = sector * self.config.get_bytes_per_sector() + local_index * DIRECTORY_ENTRY_SIZE;

    let driver = devices::get_driver_for_device(self.drive_number).ok_or(())?;
    driver.seek(self.drive_access_handle, SeekMethod::Absolute(position))?;

    {
      let mut buffer = self.io_buffer.write();
      let total_slice = buffer.as_mut_slice();
      let subset = &mut total_slice[0..DIRECTORY_ENTRY_SIZE];
      driver.read(self.drive_access_handle, subset)?;
    }

    let buffer_addr = self.get_io_buffer_address();
    let entry = DirectoryEntry::at_address(buffer_addr);

    if entry.is_empty() {
      info.file_name = [0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20];
      info.file_ext = [0x20, 0x20, 0x20];
      info.entry_type = DirEntryType::Empty;
      info.byte_size = 0;
    } else {
      entry.copy_name(&mut info.file_name);
      entry.copy_ext(&mut info.file_ext);
      info.entry_type = DirEntryType::File;
      info.byte_size = entry.get_byte_size();
    }

    Ok(())
  }
}