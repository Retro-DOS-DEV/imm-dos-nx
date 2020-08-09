use alloc::sync::Arc;
use alloc::vec::Vec;
use super::disk::{DiskConfig, SectorRange};

/// Wrapper type representing a cluster index
/// Clusters typically have a 1-1 relationship with sectors, but they may differ
/// so we want to have a special data type for them.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Cluster(usize);

impl Cluster {
  pub fn new(index: usize) -> Cluster {
    Cluster(index)
  }

  pub fn as_usize(&self) -> usize {
    self.0
  }
}

pub struct ClusterChain {
  pub clusters: Arc<Vec<Cluster>>,
}

impl ClusterChain {
  pub fn empty() -> ClusterChain {
    ClusterChain {
      clusters: Arc::new(Vec::new()),
    }
  }

  pub fn sector_iter(&self, disk_config: &DiskConfig) -> ChainSectorIterator {
    ChainSectorIterator::new(Arc::clone(&self.clusters), disk_config)
  }
}

pub struct ChainSectorIterator {
  clusters: Arc<Vec<Cluster>>,
  cluster_index: usize,
  sector_index: usize,
  sectors_per_cluster: usize,
  root_dir_sectors: SectorRange,
  data_sectors: SectorRange,
}

impl ChainSectorIterator {
  pub fn new(clusters: Arc<Vec<Cluster>>, disk_config: &DiskConfig) -> ChainSectorIterator {
    ChainSectorIterator {
      clusters,
      cluster_index: 0,
      sector_index: 0,
      sectors_per_cluster: disk_config.get_sectors_per_cluster(),
      root_dir_sectors: disk_config.get_root_directory_sectors(),
      data_sectors: disk_config.get_data_sectors(),
    }
  }
}

impl Iterator for ChainSectorIterator {
  type Item = usize;

  fn next(&mut self) -> Option<usize> {
    let cluster_count = self.clusters.len();
    if cluster_count == 0 {
      // No clusters means we're iterating over the root directory
      if self.sector_index > self.root_dir_sectors.get_sector_count() {
        return None;
      }
      let sector = self.root_dir_sectors.get_first_sector() + self.sector_index;
      return Some(sector);
    }

    if self.cluster_index >= cluster_count {
      return None;
    }
    let current_cluster = self.clusters[self.cluster_index].as_usize();
    if current_cluster < 2 {
      return None;
    }
    let cluster_start =
      self.data_sectors.get_first_sector() +
      (current_cluster - 2) * self.sectors_per_cluster;
    let sector = cluster_start + self.sector_index;
    self.sector_index += 1;
    if self.sector_index >= self.sectors_per_cluster {
      self.cluster_index += 1;
      self.sector_index = 0;
    }
    Some(sector)
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FatEntry {
  NextCluster(Cluster),
  EndOfChain,
  Free,
  BadSector,
  Reserved,
  TemporaryAllocation,
}

impl FatEntry {
  pub fn from_value(value: u16) -> FatEntry {
    match value {
      0 => FatEntry::Free,
      1 => FatEntry::TemporaryAllocation,
      0xff0..=0xff5 => FatEntry::EndOfChain,
      0xff6 => FatEntry::Reserved,
      0xff7 => FatEntry::BadSector,
      0xff8..=0xfff => FatEntry::EndOfChain,
      _ => FatEntry::NextCluster(Cluster::new(value as usize)),
    }
  }
}

pub struct FatSection<'table> {
  /// Pointer to a FAT table currently cached in memory
  section: &'table mut [u8],
  /// Offset of the first cluster in the table. FAT12 tables are not sector-
  /// aligned, so some sectors may start with the end of a previous cluster
  byte_offset: usize,
  /// Cluster ID of the first entry after byte_offset
  first_cluster: Cluster,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FatValueResult {
  /// Indicates the requested cluster comes before the start of the table
  OutOfBoundsBefore,
  /// Indicates the requested cluster comes after the end of the table
  OutOfBoundsAfter,
  /// Indicates the value wraps beyond the end of the table, and returns the
  /// lowest 8 bits of the entry
  Partial8(u8),
  /// Similar to Partial8, but returns the lower 4 bits of the entry
  Partial4(u8),
  /// Returns a FatEntry that was fully contained within the table
  Success(FatEntry), 
}

impl<'table> FatSection<'table> {
  pub fn at_slice(section: &'table mut [u8], byte_offset: usize, first_cluster: Cluster) -> FatSection<'table> {
    FatSection {
      section,
      byte_offset,
      first_cluster,
    }
  }

  pub fn get_value(&self, cluster: Cluster) -> FatValueResult {
    let target_cluster = cluster.as_usize();
    let first_cluster = self.first_cluster.as_usize();
    if target_cluster < first_cluster {
      return FatValueResult::OutOfBoundsBefore;
    }
    let distance = target_cluster - first_cluster;
    let triad_start = (distance / 2) * 3 + self.byte_offset;
    if triad_start >= self.section.len() {
      return FatValueResult::OutOfBoundsAfter;
    }
    let triad_offset = distance & 1;
    let byte_addr = triad_start + triad_offset;
    if self.section.len() - byte_addr < 2 {
      if triad_offset == 0 {
        return FatValueResult::Partial8(self.section[byte_addr]);
      }
      return FatValueResult::Partial4(self.section[byte_addr] >> 4);
    }

    let low = self.section[byte_addr];
    let high = self.section[byte_addr + 1];
    let mut value = (low as u16) | ((high as u16) << 8);
    value >>= triad_offset * 4;
    value &= 0xfff;

    FatValueResult::Success(FatEntry::from_value(value))
  }
}

#[cfg(test)]
mod tests {
  use super::{Cluster, FatEntry, FatSection, FatValueResult};

  #[test]
  fn simple_fetch() {
    let mut mem = [0xf0, 0xff, 0xff, 0x03, 0x40, 0x00, 0x05, 0xf0, 0xff, 0x00];
    let section = FatSection::at_slice(&mut mem, 0, Cluster::new(0));
    assert_eq!(section.get_value(Cluster::new(0)), FatValueResult::Success(FatEntry::EndOfChain));
    assert_eq!(section.get_value(Cluster::new(1)), FatValueResult::Success(FatEntry::EndOfChain));
    assert_eq!(section.get_value(Cluster::new(2)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(3))));
    assert_eq!(section.get_value(Cluster::new(3)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(4))));
    assert_eq!(section.get_value(Cluster::new(4)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(5))));
    assert_eq!(section.get_value(Cluster::new(5)), FatValueResult::Success(FatEntry::EndOfChain));
    assert_eq!(section.get_value(Cluster::new(6)), FatValueResult::Partial8(0));
  }

  #[test]
  fn offset_table() {
    let mut offset_one = [0x6f, 0x08, 0x90, 0x00, 0xff, 0x0f, 0x00, 0x0c, 0xf0, 0x00];
    let section_one = FatSection::at_slice(&mut offset_one, 1, Cluster::new(7));
    assert_eq!(section_one.get_value(Cluster::new(6)), FatValueResult::OutOfBoundsBefore);
    assert_eq!(section_one.get_value(Cluster::new(7)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(8))));
    assert_eq!(section_one.get_value(Cluster::new(8)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(9))));
    assert_eq!(section_one.get_value(Cluster::new(9)), FatValueResult::Success(FatEntry::EndOfChain));
    assert_eq!(section_one.get_value(Cluster::new(0xa)), FatValueResult::Success(FatEntry::Free));
    assert_eq!(section_one.get_value(Cluster::new(0xb)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(0xc))));
    assert_eq!(section_one.get_value(Cluster::new(0xc)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(0xf))));

    let mut offset_two = [0x6f, 0xff, 0xf7, 0xaf, 0x10, 0x00, 0x00, 0x00, 0x1f, 0x23];
    let section_two = FatSection::at_slice(&mut offset_two, 2, Cluster::new(0x10));
    assert_eq!(section_two.get_value(Cluster::new(0x10)), FatValueResult::Success(FatEntry::BadSector));
    assert_eq!(section_two.get_value(Cluster::new(0x11)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(0x10a))));
    assert_eq!(section_two.get_value(Cluster::new(0x14)), FatValueResult::Success(FatEntry::NextCluster(Cluster::new(0x31f))));
    assert_eq!(section_two.get_value(Cluster::new(0x15)), FatValueResult::Partial4(2));

  }
}
