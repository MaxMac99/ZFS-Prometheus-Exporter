use anyhow::{Context, Result};
use tracing::debug;

#[derive(Debug, Default)]
pub struct ArcStats {
    pub hits: u64,
    pub iohits: u64,
    pub misses: u64,
    pub size_bytes: u64,
    pub target_bytes: u64,   // c
    pub max_size_bytes: u64, // c_max
    pub min_size_bytes: u64, // c_min
    pub data_size_bytes: u64,
    pub metadata_size_bytes: u64,
    pub overhead_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub uncompressed_size_bytes: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l2_size: u64,
    pub l2_asize: u64,
    pub l2_hdr_size: u64,
    pub l2_read_bytes_total: u64,
    pub l2_write_bytes_total: u64,
}

fn parse_arc_stats(content: &str) -> Result<ArcStats> {
    let mut arc_stats = ArcStats::default();

    for line in content.lines().skip(2) {
        // Skip header lines
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let name = parts[0];
        let value = parts[2].parse::<u64>().unwrap_or(0);

        match name {
            "hits" => arc_stats.hits = value,
            "iohits" => arc_stats.iohits = value,
            "misses" => arc_stats.misses = value,
            "size" => arc_stats.size_bytes = value,
            "c" => arc_stats.target_bytes = value,
            "c_max" => arc_stats.max_size_bytes = value,
            "c_min" => arc_stats.min_size_bytes = value,
            "data_size" => arc_stats.data_size_bytes = value,
            "metadata_size" => arc_stats.metadata_size_bytes = value,
            "overhead_size" => arc_stats.overhead_size_bytes = value,
            "compressed_size" => arc_stats.compressed_size_bytes = value,
            "uncompressed_size" => arc_stats.uncompressed_size_bytes = value,
            "l2_hits" => arc_stats.l2_hits = value,
            "l2_misses" => arc_stats.l2_misses = value,
            "l2_size" => arc_stats.l2_size = value,
            "l2_asize" => arc_stats.l2_asize = value,
            "l2_hdr_size" => arc_stats.l2_hdr_size = value,
            "l2_read_bytes" => arc_stats.l2_read_bytes_total = value,
            "l2_write_bytes" => arc_stats.l2_write_bytes_total = value,
            _ => {}
        }
    }

    Ok(arc_stats)
}

pub async fn read_arc_stats() -> Result<ArcStats> {
    let arcstats_path = "/proc/spl/kstat/zfs/arcstats";
    debug!("reading arc stats at {}", arcstats_path);
    let content = tokio::fs::read_to_string(arcstats_path)
        .await
        .context("Failed to read arcstats (is ZFS loaded?)")?;
    debug!("read arc stats, now parsing");

    parse_arc_stats(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_arcstats_fixture() {
        let content = include_str!("fixtures/arcstats");
        let result: Result<ArcStats, _> = parse_arc_stats(content);

        match result {
            Ok(status) => {
                println!("Successfully deserialized ARC stats: {:?}", status);
            }
            Err(e) => {
                panic!("Failed to deserialize zpool status fixture: {}", e);
            }
        }
    }
}
