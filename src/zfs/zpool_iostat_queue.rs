use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, trace};

#[derive(Debug)]
pub struct ZpoolQueueStats {
    pub pools: HashMap<String, QueueStat>,
}

#[derive(Debug)]
pub struct QueueStat {
    pub capacity_alloc: u64,
    pub capacity_free: u64,
    pub operations_read: u64,
    pub operations_write: u64,
    pub bandwidth_read: u64,
    pub bandwidth_write: u64,
    pub sync_queue_read_pending: u64,
    pub sync_queue_read_active: u64,
    pub sync_queue_write_pending: u64,
    pub sync_queue_write_active: u64,
    pub async_queue_read_pending: u64,
    pub async_queue_read_active: u64,
    pub async_queue_write_pending: u64,
    pub async_queue_write_active: u64,
    pub scrub_queue_read_pending: u64,
    pub scrub_queue_read_active: u64,
    pub trim_queue_write_pending: u64,
    pub trim_queue_write_active: u64,
    pub rebuild_queue_write_pending: u64,
    pub rebuild_queue_write_active: u64,
}

fn parse_pool_queue_stats(content: &str) -> Result<ZpoolQueueStats> {
    let mut pools = HashMap::new();
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .try_for_each(|line| {
            let items = line.split_whitespace().collect::<Vec<_>>();

            let pool_name = *items.first().context("Could not parse name")?;
            let stats = QueueStat {
                capacity_alloc: items
                    .get(1)
                    .context("Could not find capacity_alloc")?
                    .parse()
                    .context("Could not parse capacity_alloc")?,
                capacity_free: items
                    .get(2)
                    .context("Could not find capacity_free")?
                    .parse()
                    .context("Could not parse capacity_free")?,
                operations_read: items
                    .get(3)
                    .context("Could not find operations_read")?
                    .parse()
                    .context("Could not parse operations_read")?,
                operations_write: items
                    .get(4)
                    .context("Could not find operations_write")?
                    .parse()
                    .context("Could not parse operations_write")?,
                bandwidth_read: items
                    .get(5)
                    .context("Could not find bandwidth_read")?
                    .parse()
                    .context("Could not parse bandwidth_read")?,
                bandwidth_write: items
                    .get(6)
                    .context("Could not find bandwidth_write")?
                    .parse()
                    .context("Could not parse bandwidth_write")?,
                sync_queue_read_pending: items
                    .get(7)
                    .context("Could not find sync_queue_read_pending")?
                    .parse()
                    .context("Could not parse sync_queue_read_pending")?,
                sync_queue_read_active: items
                    .get(8)
                    .context("Could not find sync_queue_read_active")?
                    .parse()
                    .context("Could not parse sync_queue_read_active")?,
                sync_queue_write_pending: items
                    .get(9)
                    .context("Could not find sync_queue_write_pending")?
                    .parse()
                    .context("Could not parse sync_queue_write_pending")?,
                sync_queue_write_active: items
                    .get(10)
                    .context("Could not find sync_queue_write_active")?
                    .parse()
                    .context("Could not parse sync_queue_write_active")?,
                async_queue_read_pending: items
                    .get(11)
                    .context("Could not find async_queue_read_pending")?
                    .parse()
                    .context("Could not parse async_queue_read_pending")?,
                async_queue_read_active: items
                    .get(12)
                    .context("Could not find async_queue_read_active")?
                    .parse()
                    .context("Could not parse async_queue_read_active")?,
                async_queue_write_pending: items
                    .get(13)
                    .context("Could not find async_queue_write_pending")?
                    .parse()
                    .context("Could not parse async_queue_write_pending")?,
                async_queue_write_active: items
                    .get(14)
                    .context("Could not find async_queue_write_active")?
                    .parse()
                    .context("Could not parse async_queue_write_active")?,
                scrub_queue_read_pending: items
                    .get(15)
                    .context("Could not find scrub_queue_read_pending")?
                    .parse()
                    .context("Could not parse scrub_queue_read_pending")?,
                scrub_queue_read_active: items
                    .get(16)
                    .context("Could not find scrub_queue_read_active")?
                    .parse()
                    .context("Could not parse scrub_queue_read_active")?,
                trim_queue_write_pending: items
                    .get(17)
                    .context("Could not find trim_queue_write_pending")?
                    .parse()
                    .context("Could not parse trim_queue_write_pending")?,
                trim_queue_write_active: items
                    .get(18)
                    .context("Could not find trim_queue_write_active")?
                    .parse()
                    .context("Could not parse trim_queue_write_active")?,
                rebuild_queue_write_pending: items
                    .get(19)
                    .context("Could not find rebuild_queue_write_pending")?
                    .parse()
                    .context("Could not parse rebuild_queue_write_pending")?,
                rebuild_queue_write_active: items
                    .get(20)
                    .context("Could not find rebuild_queue_write_active")?
                    .parse()
                    .context("Could not parse rebuild_queue_write_active")?,
            };
            pools.insert(String::from(pool_name), stats);
            Ok::<(), anyhow::Error>(())
        })?;
    Ok(ZpoolQueueStats { pools })
}

pub async fn get_pool_queue_stats() -> Result<ZpoolQueueStats> {
    debug!("Running zpool iostat -qvHp");
    let output = Command::new("zpool")
        .args(["iostat", "-qvHp"])
        .output()
        .await
        .context("Failed to execute zpool iostat -q")?;

    if !output.status.success() {
        anyhow::bail!(
            "zpool iostat -q failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    debug!("zpool iostat -qvHp command executed successfully");
    trace!("zpool iostat -qvHp output: {:?}", &output);

    let content =
        String::from_utf8(output.stdout).context("Failed to parse zpool iostat -q JSON")?;
    parse_pool_queue_stats(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_zpool_queue_fixture() {
        let json_data = include_str!("fixtures/zpool_iostat_queue.txt");
        let result: Result<ZpoolQueueStats, _> = parse_pool_queue_stats(json_data);

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
