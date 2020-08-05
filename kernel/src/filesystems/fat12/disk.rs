use super::errors::FatError;

const DIRECTORY_ENTRY_SIZE: usize = 32;

/// Wrapper type representing a cluster index
/// Clusters typically have a 1-1 relationship with sectors, but they may differ
/// so we want to have a special data type for them.
pub struct Cluster(usize);

impl Cluster {
  pub fn new(index: usize) -> Cluster {
    Cluster(index)
  }

  pub fn as_usize(&self) -> usize {
    self.0
  }
}

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
  sectors_per_cluster: usize,
  reserved_sectors: usize,
  fat_count: usize,
  root_directory_entries: usize,
  sectors_per_fat: usize,
}

impl DiskConfig {
  pub fn empty() -> DiskConfig {
    DiskConfig {
      sectors_per_cluster: 1,
      reserved_sectors: 1,
      fat_count: 1,
      root_directory_entries: 1,
      sectors_per_fat: 1,
    }
  }

  pub fn from_bpb(&mut self, bpb: &BiosParamBlock) {
    self.sectors_per_cluster = bpb.sectors_per_cluster as usize;
    self.reserved_sectors = bpb.reserved_sectors as usize;
    self.fat_count = bpb.fat_count as usize;
    self.root_directory_entries = bpb.root_directory_entries as usize;
    self.sectors_per_fat = bpb.sectors_per_fat as usize;
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
