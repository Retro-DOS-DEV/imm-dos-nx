use alloc::sync::Arc;
use alloc::vec::Vec;
use super::disk::{DiskConfig, SectorRange};

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
