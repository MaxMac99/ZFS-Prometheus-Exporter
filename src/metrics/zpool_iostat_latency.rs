use crate::zfs::zpool_iostat_latency::{get_pool_latency_stats, LatencyStat};
use anyhow::Result;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

pub struct ZpoolLatencyMetrics {
    pub zpool_latency_operations: Family<LatencyLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct LatencyLabels {
    pub pool: String,
    pub latency_ns: String,
    pub operation: String,
}

impl ZpoolLatencyMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        let zpool_latency_operations = Family::<LatencyLabels, Gauge>::default();
        registry.register(
            "zpool_latency_operations_total",
            "Number of operations by latency bucket and operation type",
            zpool_latency_operations.clone(),
        );

        ZpoolLatencyMetrics {
            zpool_latency_operations,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let latency_stats = get_pool_latency_stats().await?;

        for (pool_name, stats) in latency_stats.pools {
            for stat in stats {
                self.collect_latency_stat(&pool_name, &stat);
            }
        }

        Ok(())
    }

    fn collect_latency_stat(&self, pool_name: &str, stat: &LatencyStat) {
        let latency = stat.latency.to_string();

        // Total wait operations
        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "total_wait_read"))
            .set(stat.total_wait_read as i64);

        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "total_wait_write"))
            .set(stat.total_wait_write as i64);

        // Disk wait operations
        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "disk_wait_read"))
            .set(stat.disk_wait_read as i64);

        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "disk_wait_write"))
            .set(stat.disk_wait_write as i64);

        // Sync queue wait operations
        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "syncq_wait_read"))
            .set(stat.syncq_wait_read as i64);

        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "syncq_wait_write"))
            .set(stat.syncq_wait_write as i64);

        // Async queue wait operations
        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "asyncq_wait_read"))
            .set(stat.asyncq_wait_read as i64);

        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(
                pool_name,
                &latency,
                "asyncq_wait_write",
            ))
            .set(stat.asyncq_wait_write as i64);

        // Scrub operations
        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "scrub"))
            .set(stat.scrub as i64);

        // Trim operations
        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "trim"))
            .set(stat.trim as i64);

        // Rebuild operations
        self.zpool_latency_operations
            .get_or_create(&LatencyLabels::new(pool_name, &latency, "rebuild"))
            .set(stat.rebuild as i64);
    }
}

impl LatencyLabels {
    pub fn new(pool: &str, latency_ns: &str, operation: &str) -> Self {
        LatencyLabels {
            pool: pool.to_string(),
            latency_ns: latency_ns.to_string(),
            operation: operation.to_string(),
        }
    }
}
