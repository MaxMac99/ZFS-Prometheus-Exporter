use crate::metrics::arc::ArcMetrics;
use crate::metrics::zfs_list::ZfsListMetrics;
use crate::metrics::zpool_iostat_io_size::ZpoolIoSizeMetrics;
use crate::metrics::zpool_iostat_latency::ZpoolLatencyMetrics;
use crate::metrics::zpool_iostat_queue::ZpoolQueueMetrics;
use crate::metrics::zpool_list::ZpoolListMetrics;
use crate::metrics::zpool_status::ZpoolStatusMetrics;
use anyhow::Result;
use prometheus_client::registry::Registry;
use tracing::{debug, error, warn};

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
        debug!("collecting metrics");
        let (status_res, list_res, io_size_res, latency_res, queue_res, zfs_res, arc_res) = tokio::join!(
            self.zpool_status.collect(),
            self.zpool_list.collect(),
            self.zpool_io_size.collect(),
            self.zpool_latency.collect(),
            self.zpool_queue.collect(),
            self.zfs_list.collect(),
            self.arc.collect(),
        );

        let mut errors = Vec::new();

        if let Err(e) = status_res {
            error!("Failed to collect zpool status metrics: {:?}", e);
            errors.push(format!("zpool_status: {}", e));
        }
        if let Err(e) = list_res {
            error!("Failed to collect zpool list metrics: {:?}", e);
            errors.push(format!("zpool_list: {}", e));
        }
        if let Err(e) = io_size_res {
            error!("Failed to collect zpool IO size metrics: {:?}", e);
            errors.push(format!("zpool_io_size: {}", e));
        }
        if let Err(e) = latency_res {
            error!("Failed to collect zpool latency metrics: {:?}", e);
            errors.push(format!("zpool_latency: {}", e));
        }
        if let Err(e) = queue_res {
            error!("Failed to collect zpool queue metrics: {:?}", e);
            errors.push(format!("zpool_queue: {}", e));
        }
        if let Err(e) = zfs_res {
            error!("Failed to collect zfs list metrics: {:?}", e);
            errors.push(format!("zfs_list: {}", e));
        }
        if let Err(e) = arc_res {
            error!("Failed to collect ARC metrics: {:?}", e);
            errors.push(format!("arc: {}", e));
        }

        if !errors.is_empty() {
            warn!("{} metric collection(s) failed", errors.len());
        }

        Ok(())
    }
}
