use crate::zfs::zpool_iostat_queue::{get_pool_queue_stats, QueueStat};
use anyhow::Result;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

pub struct ZpoolQueueMetrics {
    pub zpool_capacity_bytes: Family<ZpoolLabels, Gauge>,
    pub zpool_operations: Family<ZpoolOperationLabels, Gauge>,
    pub zpool_bandwidth_bytes: Family<ZpoolOperationLabels, Gauge>,
    pub zpool_queue: Family<ZpoolQueueLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolLabels {
    pub pool: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolOperationLabels {
    pub pool: String,
    pub operation: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolQueueLabels {
    pub pool: String,
    pub queue_type: String,
    pub operation: String,
    pub state: String,
}

impl ZpoolQueueMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        let zpool_capacity_bytes = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_queue_capacity_bytes",
            "ZFS pool capacity in bytes (allocated/free)",
            zpool_capacity_bytes.clone(),
        );

        let zpool_operations = Family::<ZpoolOperationLabels, Gauge>::default();
        registry.register(
            "zpool_queue_operations_total",
            "Number of operations by type",
            zpool_operations.clone(),
        );

        let zpool_bandwidth_bytes = Family::<ZpoolOperationLabels, Gauge>::default();
        registry.register(
            "zpool_queue_bandwidth_bytes",
            "Bandwidth in bytes by operation type",
            zpool_bandwidth_bytes.clone(),
        );

        let zpool_queue = Family::<ZpoolQueueLabels, Gauge>::default();
        registry.register(
            "zpool_queue_depth",
            "Queue depth for different queue types and operations",
            zpool_queue.clone(),
        );

        ZpoolQueueMetrics {
            zpool_capacity_bytes,
            zpool_operations,
            zpool_bandwidth_bytes,
            zpool_queue,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let queue_stats = get_pool_queue_stats().await?;

        for (pool_name, stat) in queue_stats.pools {
            self.collect_queue_stat(&pool_name, &stat);
        }

        Ok(())
    }

    fn collect_queue_stat(&self, pool_name: &str, stat: &QueueStat) {
        // Capacity metrics
        self.zpool_capacity_bytes
            .get_or_create(&ZpoolLabels::new(pool_name))
            .set(stat.capacity_alloc as i64);

        self.zpool_capacity_bytes
            .get_or_create(&ZpoolLabels::new(pool_name))
            .set(stat.capacity_free as i64);

        // Operations
        self.zpool_operations
            .get_or_create(&ZpoolOperationLabels::new(pool_name, "read"))
            .set(stat.operations_read as i64);

        self.zpool_operations
            .get_or_create(&ZpoolOperationLabels::new(pool_name, "write"))
            .set(stat.operations_write as i64);

        // Bandwidth
        self.zpool_bandwidth_bytes
            .get_or_create(&ZpoolOperationLabels::new(pool_name, "read"))
            .set(stat.bandwidth_read as i64);

        self.zpool_bandwidth_bytes
            .get_or_create(&ZpoolOperationLabels::new(pool_name, "write"))
            .set(stat.bandwidth_write as i64);

        // Sync queue
        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(pool_name, "sync", "read", "pending"))
            .set(stat.sync_queue_read_pending as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(pool_name, "sync", "read", "active"))
            .set(stat.sync_queue_read_active as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "sync", "write", "pending",
            ))
            .set(stat.sync_queue_write_pending as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(pool_name, "sync", "write", "active"))
            .set(stat.sync_queue_write_active as i64);

        // Async queue
        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "async", "read", "pending",
            ))
            .set(stat.async_queue_read_pending as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(pool_name, "async", "read", "active"))
            .set(stat.async_queue_read_active as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "async", "write", "pending",
            ))
            .set(stat.async_queue_write_pending as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "async", "write", "active",
            ))
            .set(stat.async_queue_write_active as i64);

        // Scrub queue
        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "scrub", "read", "pending",
            ))
            .set(stat.scrub_queue_read_pending as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(pool_name, "scrub", "read", "active"))
            .set(stat.scrub_queue_read_active as i64);

        // Trim queue
        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "trim", "write", "pending",
            ))
            .set(stat.trim_queue_write_pending as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(pool_name, "trim", "write", "active"))
            .set(stat.trim_queue_write_active as i64);

        // Rebuild queue
        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "rebuild", "write", "pending",
            ))
            .set(stat.rebuild_queue_write_pending as i64);

        self.zpool_queue
            .get_or_create(&ZpoolQueueLabels::new(
                pool_name, "rebuild", "write", "active",
            ))
            .set(stat.rebuild_queue_write_active as i64);
    }
}

impl ZpoolLabels {
    pub fn new(pool: &str) -> Self {
        ZpoolLabels {
            pool: pool.to_string(),
        }
    }
}

impl ZpoolOperationLabels {
    pub fn new(pool: &str, operation: &str) -> Self {
        ZpoolOperationLabels {
            pool: pool.to_string(),
            operation: operation.to_string(),
        }
    }
}

impl ZpoolQueueLabels {
    pub fn new(pool: &str, queue_type: &str, operation: &str, state: &str) -> Self {
        ZpoolQueueLabels {
            pool: pool.to_string(),
            queue_type: queue_type.to_string(),
            operation: operation.to_string(),
            state: state.to_string(),
        }
    }
}
