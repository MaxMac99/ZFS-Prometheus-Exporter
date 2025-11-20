use crate::metrics::arc::ArcMetrics;
use crate::metrics::zfs_list::ZfsListMetrics;
use crate::metrics::zpool_iostat_io_size::ZpoolIoSizeMetrics;
use crate::metrics::zpool_iostat_latency::ZpoolLatencyMetrics;
use crate::metrics::zpool_iostat_queue::ZpoolQueueMetrics;
use crate::metrics::zpool_list::ZpoolListMetrics;
use crate::metrics::zpool_status::ZpoolStatusMetrics;
use anyhow::Result;
use prometheus_client::registry::Registry;

mod arc;
pub mod handlers;
mod zfs_list;
mod zpool_iostat_io_size;
mod zpool_iostat_latency;
mod zpool_iostat_queue;
mod zpool_list;
mod zpool_status;

pub struct Metrics {
    pub registry: Registry,
    zpool_status: ZpoolStatusMetrics,
    zpool_list: ZpoolListMetrics,
    zpool_io_size: ZpoolIoSizeMetrics,
    zpool_latency: ZpoolLatencyMetrics,
    zpool_queue: ZpoolQueueMetrics,
    zfs_list: ZfsListMetrics,
    arc: ArcMetrics,
}

impl Metrics {
    pub fn new() -> Self {
        let mut registry = Registry::default();
        let zpool_status = ZpoolStatusMetrics::new(&mut registry);
        let zpool_list = ZpoolListMetrics::new(&mut registry);
        let zpool_io_size = ZpoolIoSizeMetrics::new(&mut registry);
        let zpool_latency = ZpoolLatencyMetrics::new(&mut registry);
        let zpool_queue = ZpoolQueueMetrics::new(&mut registry);
        let zfs_list = ZfsListMetrics::new(&mut registry);
        let arc = ArcMetrics::new(&mut registry);
        Self {
            registry,
            zpool_status,
            zpool_list,
            zpool_io_size,
            zpool_latency,
            zpool_queue,
            zfs_list,
            arc,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        tokio::try_join!(
            self.zpool_status.collect(),
            self.zpool_list.collect(),
            self.zpool_io_size.collect(),
            self.zpool_latency.collect(),
            self.zpool_queue.collect(),
            self.zfs_list.collect(),
            self.arc.collect(),
        )?;
        Ok(())
    }
}
