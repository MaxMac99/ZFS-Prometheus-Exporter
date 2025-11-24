use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, trace};

#[derive(Debug)]
pub struct ZpoolLatencyStats {
    pub pools: HashMap<String, Vec<LatencyStat>>,
}

#[derive(Debug)]
pub struct LatencyStat {
    pub latency: u64, // ns
    pub total_wait_read: u64,
    pub total_wait_write: u64,
    pub disk_wait_read: u64,
    pub disk_wait_write: u64,
    pub syncq_wait_read: u64,
    pub syncq_wait_write: u64,
    pub asyncq_wait_read: u64,
    pub asyncq_wait_write: u64,
    pub scrub: u64,
    pub trim: u64,
    pub rebuild: u64,
}

fn parse_pool_latency_stats(content: &str) -> Result<ZpoolLatencyStats> {
    let mut pools = HashMap::new();
    content.split("\n\n").try_for_each(|section| {
        let mut lines = section.trim_start().lines();
        let pool_name = lines.next().context("Could not parse name")?;

        let stats = lines
            .map(|line| {
                let items = line.split_whitespace().collect::<Vec<_>>();
                Ok(LatencyStat {
                    latency: items
                        .first()
                        .context("Could not find latency")?
                        .parse()
                        .context("Could not parse latency")?,
                    total_wait_read: items
                        .get(1)
                        .context("Could not find total_wait_read")?
                        .parse()
                        .context("Could not parse total_wait_read")?,
                    total_wait_write: items
                        .get(2)
                        .context("Could not find total_wait_write")?
                        .parse()
                        .context("Could not parse total_wait_write")?,
                    disk_wait_read: items
                        .get(3)
                        .context("Could not find disk_wait_read")?
                        .parse()
                        .context("Could not parse disk_wait_read")?,
                    disk_wait_write: items
                        .get(4)
                        .context("Could not find disk_wait_write")?
                        .parse()
                        .context("Could not parse disk_wait_write")?,
                    syncq_wait_read: items
                        .get(5)
                        .context("Could not find syncq_wait_read")?
                        .parse()
                        .context("Could not parse syncq_wait_read")?,
                    syncq_wait_write: items
                        .get(6)
                        .context("Could not find syncq_wait_write")?
                        .parse()
                        .context("Could not parse syncq_wait_write")?,
                    asyncq_wait_read: items
                        .get(7)
                        .context("Could not find asyncq_wait_read")?
                        .parse()
                        .context("Could not parse asyncq_wait_read")?,
                    asyncq_wait_write: items
                        .get(8)
                        .context("Could not find asyncq_wait_write")?
                        .parse()
                        .context("Could not parse asyncq_wait_write")?,
                    scrub: items
                        .get(9)
                        .context("Could not find scrub")?
                        .parse()
                        .context("Could not parse scrub")?,
                    trim: items
                        .get(10)
                        .context("Could not find trim")?
                        .parse()
                        .context("Could not parse trim")?,
                    rebuild: items
                        .get(11)
                        .context("Could not find rebuild")?
                        .parse()
                        .context("Could not parse rebuild")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        pools.insert(String::from(pool_name), stats);
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(ZpoolLatencyStats { pools })
}

pub async fn get_pool_latency_stats() -> Result<ZpoolLatencyStats> {
    debug!("Running zpool iostat -wvHp to get pool latency stats");
    let output = Command::new("zpool")
        .args(["iostat", "-wvHp"])
        .output()
        .await
        .context("Failed to execute zpool iostat -w")?;

    if !output.status.success() {
        anyhow::bail!(
            "zpool iostat -w failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    debug!("zpool iostat -wvHp command executed successfully");
    trace!("zpool iostat -wvHp output: {:?}", &output);

    let content =
        String::from_utf8(output.stdout).context("Failed to parse zpool iostat -w JSON")?;
    parse_pool_latency_stats(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_zpool_latency_fixture() {
        let json_data = include_str!("fixtures/zpool_iostat_latency.txt");
        let result: Result<ZpoolLatencyStats, _> = parse_pool_latency_stats(json_data);

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
