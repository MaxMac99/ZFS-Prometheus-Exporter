use crate::zfs::zpool_iostat_io_size::{get_pool_io_size_stats, IoSizeStat};
use anyhow::Result;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

pub struct ZpoolIoSizeMetrics {
    pub zpool_io_size_operations: Family<IoSizeLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct IoSizeLabels {
    pub pool: String,
    pub req_size: String,
    pub operation: String,
    pub io_type: String,
}

impl ZpoolIoSizeMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        let zpool_io_size_operations = Family::<IoSizeLabels, Gauge>::default();
        registry.register(
            "zpool_io_size_operations_total",
            "Number of I/O operations by size and type",
            zpool_io_size_operations.clone(),
        );

        ZpoolIoSizeMetrics {
            zpool_io_size_operations,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let io_size_stats = get_pool_io_size_stats().await?;

        for (pool_name, stats) in io_size_stats.pools {
            for stat in stats {
                self.collect_io_size_stat(&pool_name, &stat);
            }
        }

        Ok(())
    }

    fn collect_io_size_stat(&self, pool_name: &str, stat: &IoSizeStat) {
        let req_size = stat.req_size.to_string();

        // Sync Read
        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "sync_read",
                "independent",
            ))
            .set(stat.sync_read_independent as i64);

        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "sync_read",
                "aggregated",
            ))
            .set(stat.sync_read_aggregated as i64);

        // Sync Write
        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "sync_write",
                "independent",
            ))
            .set(stat.sync_write_independent as i64);

        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "sync_write",
                "aggregated",
            ))
            .set(stat.sync_write_aggregated as i64);

        // Async Read
        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "async_read",
                "independent",
            ))
            .set(stat.async_read_independent as i64);

        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "async_read",
                "aggregated",
            ))
            .set(stat.async_read_aggregated as i64);

        // Async Write
        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "async_write",
                "independent",
            ))
            .set(stat.async_write_independent as i64);

        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "async_write",
                "aggregated",
            ))
            .set(stat.async_write_aggregated as i64);

        // Scrub
        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "scrub_read",
                "independent",
            ))
            .set(stat.scrub_read_independent as i64);

        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "scrub_read",
                "aggregated",
            ))
            .set(stat.scrub_read_aggregated as i64);

        // Trim
        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "trim_write",
                "independent",
            ))
            .set(stat.trim_write_independent as i64);

        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "trim_write",
                "aggregated",
            ))
            .set(stat.trim_write_aggregated as i64);

        // Rebuild
        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "rebuild_write",
                "independent",
            ))
            .set(stat.rebuild_write_independent as i64);

        self.zpool_io_size_operations
            .get_or_create(&IoSizeLabels::new(
                pool_name,
                &req_size,
                "rebuild_write",
                "aggregated",
            ))
            .set(stat.rebuild_write_aggregated as i64);
    }
}

impl IoSizeLabels {
    pub fn new(pool: &str, req_size: &str, operation: &str, io_type: &str) -> Self {
        IoSizeLabels {
            pool: pool.to_string(),
            req_size: req_size.to_string(),
            operation: operation.to_string(),
            io_type: io_type.to_string(),
        }
    }
}
