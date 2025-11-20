use crate::zfs::zpool_list::{get_pool_list, Pool};
use crate::zfs::zpool_status::PoolState;
use anyhow::Result;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;
use strum::IntoEnumIterator;

pub struct ZpoolListMetrics {
    pub zpool_state: Family<ZpoolStateLabels, Gauge>,
    pub zpool_size_bytes: Family<ZpoolLabels, Gauge>,
    pub zpool_allocated_bytes: Family<ZpoolLabels, Gauge>,
    pub zpool_free_bytes: Family<ZpoolLabels, Gauge>,
    pub zpool_checkpoint_bytes: Family<ZpoolLabels, Gauge>,
    pub zpool_expandsize_bytes: Family<ZpoolLabels, Gauge>,
    pub zpool_fragmentation_percent: Family<ZpoolLabels, Gauge>,
    pub zpool_capacity_percent: Family<ZpoolLabels, Gauge>,
    pub zpool_dedupratio: Family<ZpoolLabels, Gauge<f64, AtomicU64>>,
    pub zpool_health: Family<ZpoolHealthLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolLabels {
    pub pool: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolStateLabels {
    pub pool: String,
    pub state: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolHealthLabels {
    pub pool: String,
    pub health: String,
}

impl ZpoolListMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        let zpool_state = Family::<ZpoolStateLabels, Gauge>::default();
        registry.register(
            "zpool_state",
            "ZFS pool state (1 = current state, 0 = other states)",
            zpool_state.clone(),
        );

        let zpool_size_bytes = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_size_bytes",
            "Total size of ZFS pool in bytes",
            zpool_size_bytes.clone(),
        );

        let zpool_allocated_bytes = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_allocated_bytes",
            "Allocated space in ZFS pool in bytes",
            zpool_allocated_bytes.clone(),
        );

        let zpool_free_bytes = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_free_bytes",
            "Free space in ZFS pool in bytes",
            zpool_free_bytes.clone(),
        );

        let zpool_checkpoint_bytes = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_checkpoint_bytes",
            "Checkpoint space in ZFS pool in bytes",
            zpool_checkpoint_bytes.clone(),
        );

        let zpool_expandsize_bytes = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_expandsize_bytes",
            "Expandable size in ZFS pool in bytes",
            zpool_expandsize_bytes.clone(),
        );

        let zpool_fragmentation_percent = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_fragmentation_percent",
            "Fragmentation percentage of ZFS pool",
            zpool_fragmentation_percent.clone(),
        );

        let zpool_capacity_percent = Family::<ZpoolLabels, Gauge>::default();
        registry.register(
            "zpool_capacity_percent",
            "Capacity percentage of ZFS pool",
            zpool_capacity_percent.clone(),
        );

        let zpool_dedupratio = Family::<ZpoolLabels, Gauge<f64, AtomicU64>>::default();
        registry.register(
            "zpool_dedupratio",
            "Deduplication ratio of ZFS pool",
            zpool_dedupratio.clone(),
        );

        let zpool_health = Family::<ZpoolHealthLabels, Gauge>::default();
        registry.register(
            "zpool_health",
            "ZFS pool health status (1 = current health, 0 = other health states)",
            zpool_health.clone(),
        );

        ZpoolListMetrics {
            zpool_state,
            zpool_size_bytes,
            zpool_allocated_bytes,
            zpool_free_bytes,
            zpool_checkpoint_bytes,
            zpool_expandsize_bytes,
            zpool_fragmentation_percent,
            zpool_capacity_percent,
            zpool_dedupratio,
            zpool_health,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let pool_list = get_pool_list().await?;

        for (_, pool) in pool_list.pools {
            self.collect_pool_metrics(&pool);
        }

        Ok(())
    }

    fn collect_pool_metrics(&self, pool: &Pool) {
        // Pool state
        PoolState::iter().for_each(|state| {
            let labels = ZpoolStateLabels::new(pool, state);
            let gauge = self.zpool_state.get_or_create(&labels);
            if pool.state == state {
                gauge.set(1);
            } else {
                gauge.set(0);
            }
        });

        let pool_labels = ZpoolLabels::new(pool);

        // Size metrics
        if let Some(size) = pool.properties.size {
            self.zpool_size_bytes
                .get_or_create(&pool_labels)
                .set(size as i64);
        }

        if let Some(allocated) = pool.properties.allocated {
            self.zpool_allocated_bytes
                .get_or_create(&pool_labels)
                .set(allocated as i64);
        }

        if let Some(free) = pool.properties.free {
            self.zpool_free_bytes
                .get_or_create(&pool_labels)
                .set(free as i64);
        }

        if let Some(checkpoint) = pool.properties.checkpoint {
            self.zpool_checkpoint_bytes
                .get_or_create(&pool_labels)
                .set(checkpoint as i64);
        }

        if let Some(expandsize) = pool.properties.expandsize {
            self.zpool_expandsize_bytes
                .get_or_create(&pool_labels)
                .set(expandsize as i64);
        }

        if let Some(fragmentation) = pool.properties.fragmentation {
            self.zpool_fragmentation_percent
                .get_or_create(&pool_labels)
                .set(fragmentation as i64);
        }

        if let Some(capacity) = pool.properties.capacity {
            self.zpool_capacity_percent
                .get_or_create(&pool_labels)
                .set(capacity as i64);
        }

        if let Some(dedupratio) = pool.properties.dedupratio {
            self.zpool_dedupratio
                .get_or_create(&pool_labels)
                .set(dedupratio);
        }

        // Health status
        if let Some(health) = pool.properties.health {
            PoolState::iter().for_each(|state| {
                let labels = ZpoolHealthLabels::new(pool, state);
                let gauge = self.zpool_health.get_or_create(&labels);
                if health == state {
                    gauge.set(1);
                } else {
                    gauge.set(0);
                }
            });
        }
    }
}

impl ZpoolLabels {
    pub fn new(pool: &Pool) -> Self {
        ZpoolLabels {
            pool: pool.name.clone(),
        }
    }
}

impl ZpoolStateLabels {
    pub fn new(pool: &Pool, state: PoolState) -> Self {
        ZpoolStateLabels {
            pool: pool.name.clone(),
            state: state.to_string().to_lowercase(),
        }
    }
}

impl ZpoolHealthLabels {
    pub fn new(pool: &Pool, health: PoolState) -> Self {
        ZpoolHealthLabels {
            pool: pool.name.clone(),
            health: health.to_string().to_lowercase(),
        }
    }
}
