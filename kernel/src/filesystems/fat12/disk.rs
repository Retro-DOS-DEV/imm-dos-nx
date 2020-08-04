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