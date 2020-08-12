use super::errors::FatError;
use super::fat::Cluster;

pub const DIRECTORY_ENTRY_SIZE: usize = 32;

/// Represent a contiguous block of sectors on a disk
pub struct SectorRange {
  first: usize,
  count: usize,
}

impl SectorRange {
  pub fn new(first: usize, count: usize) -> SectorRange {
    SectorRange {
      first,
      count,
    }
  }

  pub fn get_first_sector(&self) -> usize {
    self.first
  }

  pub fn get_sector_count(&self) -> usize {
    self.count
  }
}

pub struct DiskConfig {
  bytes_per_sector: usize,
  sectors_per_cluster: usize,
  reserved_sectors: usize,
  fat_count: usize,
  root_directory_entries: usize,
  sectors_per_fat: usize,
  total_sectors: usize,
}

impl DiskConfig {
  pub fn empty() -> DiskConfig {
    DiskConfig {
      bytes_per_sector: 512,
      sectors_per_cluster: 1,
      reserved_sectors: 1,
      fat_count: 1,
      root_directory_entries: 1,
      sectors_per_fat: 1,
      total_sectors: 1,
    }
  }

  pub fn from_bpb(&mut self, bpb: &BiosParamBlock) {
    self.bytes_per_sector = bpb.bytes_per_sector as usize;
    self.sectors_per_cluster = bpb.sectors_per_cluster as usize;
    self.reserved_sectors = bpb.reserved_sectors as usize;
    self.fat_count = bpb.fat_count as usize;
    self.root_directory_entries = bpb.root_directory_entries as usize;
    self.sectors_per_fat = bpb.sectors_per_fat as usize;
    self.total_sectors = bpb.total_sectors as usize;
  }

  pub fn get_sectors_per_cluster(&self) -> usize {
    self.sectors_per_cluster
  }

  /// Determine which disk sectors correspond with a given cluster
  pub fn get_sectors_for_cluster(&self, cluster: Cluster) -> SectorRange {
    let first = cluster.as_usize() * self.sectors_per_cluster;
    let count = self.sectors_per_cluster;
    SectorRange::new(first, count)
  }

  /// Get the sector range associated with a specific FAT table. If that table
  /// does not exist on the disk, a FatError will be returned instead.
  pub fn get_fat_sectors(&self, fat_table: usize) -> Result<SectorRange, FatError> {
    if fat_table >= self.fat_count {
      return Err(FatError::InvalidFatTable);
    }
    let mut fat_start = self.reserved_sectors;
    fat_start += fat_table * self.sectors_per_fat;

    Ok(SectorRange::new(fat_start, self.sectors_per_fat))
  }

  pub fn get_root_directory_size(&self) -> usize {
    self.root_directory_entries * DIRECTORY_ENTRY_SIZE
  }

  pub fn get_bytes_per_sector(&self) -> usize {
    self.bytes_per_sector
  }

  pub fn get_sectors_per_fat(&self) -> usize {
    self.sectors_per_fat
  }

  pub fn get_root_directory_sectors(&self) -> SectorRange {
    let sector_count = self.get_root_directory_size() / self.bytes_per_sector;
    let first_sector = self.reserved_sectors + (self.fat_count * self.sectors_per_fat);
    SectorRange::new(first_sector, sector_count)
  }

  pub fn get_data_sectors(&self) -> SectorRange {
    let first_sector = self.get_root_directory_sectors().get_first_sector();
    let count = self.total_sectors - first_sector;
    SectorRange::new(first_sector, count)
  }

  pub fn get_directory_index_location(&self, index: usize) -> (usize, usize) {
    let entries_per_sector = self.bytes_per_sector / DIRECTORY_ENTRY_SIZE;
    let absolute_sector = index / entries_per_sector;
    let local_index = index % entries_per_sector;
    (absolute_sector, local_index)
  }
}

#[repr(C, packed)]
pub struct BiosParamBlock {
  pub bytes_per_sector: u16,
  pub sectors_per_cluster: u8,
  pub reserved_sectors: u16,
  pub fat_count: u8,
  pub root_directory_entries: u16,
  pub total_sectors: u16,
  pub media_desc: u8,
  pub sectors_per_fat: u16,
}

impl BiosParamBlock {
  pub fn empty() -> BiosParamBlock {
    BiosParamBlock {
      bytes_per_sector: 0,
      sectors_per_cluster: 0,
      reserved_sectors: 0,
      fat_count: 0,
      root_directory_entries: 0,
      total_sectors: 0,
      media_desc: 0,
      sectors_per_fat: 0,
    }
  }

  pub fn as_buffer(&mut self) -> &mut [u8] {
    let len = core::mem::size_of::<BiosParamBlock>();
    unsafe {
      let ptr = self as *mut BiosParamBlock as *mut u8;
      core::slice::from_raw_parts_mut(ptr, len)
    }
  }
}
