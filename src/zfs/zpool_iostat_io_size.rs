use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, trace};

#[derive(Debug)]
pub struct ZpoolIoSizeStats {
    pub pools: HashMap<String, Vec<IoSizeStat>>,
}

#[derive(Debug)]
pub struct IoSizeStat {
    pub req_size: u64,
    pub sync_read_independent: u64,
    pub sync_read_aggregated: u64,
    pub sync_write_independent: u64,
    pub sync_write_aggregated: u64,
    pub async_read_independent: u64,
    pub async_read_aggregated: u64,
    pub async_write_independent: u64,
    pub async_write_aggregated: u64,
    pub scrub_read_independent: u64,
    pub scrub_read_aggregated: u64,
    pub trim_write_independent: u64,
    pub trim_write_aggregated: u64,
    pub rebuild_write_independent: u64,
    pub rebuild_write_aggregated: u64,
}

fn parse_pool_io_size_stats(content: &str) -> Result<ZpoolIoSizeStats> {
    let mut pools = HashMap::new();
    content.split("\n\n").try_for_each(|section| {
        let mut lines = section.trim_start().lines();
        let pool_name = lines.next().context("Could not parse name")?;

        let stats = lines
            .map(|line| {
                let items = line.split_whitespace().collect::<Vec<_>>();
                Ok(IoSizeStat {
                    req_size: items
                        .first()
                        .context("Could not find req_size")?
                        .parse()
                        .context("Could not parse req_size")?,
                    sync_read_independent: items
                        .get(1)
                        .context("Could not find sync_read_independent")?
                        .parse()
                        .context("Could not parse sync_read_independent")?,
                    sync_read_aggregated: items
                        .get(2)
                        .context("Could not find sync_read_aggregated")?
                        .parse()
                        .context("Could not parse sync_read_aggregated")?,
                    sync_write_independent: items
                        .get(3)
                        .context("Could not find sync_write_independent")?
                        .parse()
                        .context("Could not parse sync_write_independent")?,
                    sync_write_aggregated: items
                        .get(4)
                        .context("Could not find sync_write_aggregated")?
                        .parse()
                        .context("Could not parse sync_write_aggregated")?,
                    async_read_independent: items
                        .get(5)
                        .context("Could not find async_read_independent")?
                        .parse()
                        .context("Could not parse async_read_independent")?,
                    async_read_aggregated: items
                        .get(6)
                        .context("Could not find async_read_aggregated")?
                        .parse()
                        .context("Could not parse async_read_aggregated")?,
                    async_write_independent: items
                        .get(7)
                        .context("Could not find async_write_independent")?
                        .parse()
                        .context("Could not parse async_write_independent")?,
                    async_write_aggregated: items
                        .get(8)
                        .context("Could not find async_write_aggregated")?
                        .parse()
                        .context("Could not parse async_write_aggregated")?,
                    scrub_read_independent: items
                        .get(9)
                        .context("Could not find scrub_read_independent")?
                        .parse()
                        .context("Could not parse scrub_read_independent")?,
                    scrub_read_aggregated: items
                        .get(10)
                        .context("Could not find scrub_read_aggregated")?
                        .parse()
                        .context("Could not parse scrub_read_aggregated")?,
                    trim_write_independent: items
                        .get(11)
                        .context("Could not find trim_write_independent")?
                        .parse()
                        .context("Could not parse trim_write_independent")?,
                    trim_write_aggregated: items
                        .get(12)
                        .context("Could not find trim_write_aggregated")?
                        .parse()
                        .context("Could not parse trim_write_aggregated")?,
                    rebuild_write_independent: items
                        .get(13)
                        .context("Could not find rebuild_write_independent")?
                        .parse()
                        .context("Could not parse rebuild_write_independent")?,
                    rebuild_write_aggregated: items
                        .get(14)
                        .context("Could not find rebuild_write_aggregated")?
                        .parse()
                        .context("Could not parse rebuild_write_aggregated")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        pools.insert(String::from(pool_name), stats);
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(ZpoolIoSizeStats { pools })
}

pub async fn get_pool_io_size_stats() -> Result<ZpoolIoSizeStats> {
    debug!("Running zpool iostat -rvHp");
    let output = Command::new("zpool")
        .args(["iostat", "-rvHp"])
        .output()
        .await
        .context("Failed to execute zpool iostat -r")?;

    if !output.status.success() {
        anyhow::bail!(
            "zpool iostat -r failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    debug!("zpool iostat -rvHp command executed successfully");
    trace!("zpool iostat -rvHp output: {:?}", &output);

    let content =
        String::from_utf8(output.stdout).context("Failed to parse zpool iostat -r JSON")?;
    parse_pool_io_size_stats(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_zpool_list_fixture() {
        let json_data = include_str!("fixtures/zpool_iostat_io_size.txt");
        let result: Result<ZpoolIoSizeStats, _> = parse_pool_io_size_stats(json_data);

        match result {
            Ok(status) => {
                println!("Successfully deserialized {} pools", status.pools.len());
                for pool in &status.pools {
                    println!("Pool: {:?}", pool);
                }
            }
            Err(e) => {
                panic!("Failed to deserialize zpool status fixture: {}", e);
            }
        }
    }
}
