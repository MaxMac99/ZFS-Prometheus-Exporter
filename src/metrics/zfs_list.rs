use crate::zfs::zfs_list::{get_dataset_list, Dataset};
use anyhow::Result;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

pub struct ZfsListMetrics {
    pub zfs_dataset_used_bytes: Family<DatasetLabels, Gauge>,
    pub zfs_dataset_available_bytes: Family<DatasetLabels, Gauge>,
    pub zfs_dataset_referenced_bytes: Family<DatasetLabels, Gauge>,
    pub zfs_dataset_mounted: Family<DatasetLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DatasetLabels {
    pub dataset: String,
    pub dataset_type: String,
    pub pool: String,
}

impl ZfsListMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        let zfs_dataset_used_bytes = Family::<DatasetLabels, Gauge>::default();
        registry.register(
            "zfs_dataset_used_bytes",
            "Used space in ZFS dataset in bytes",
            zfs_dataset_used_bytes.clone(),
        );

        let zfs_dataset_available_bytes = Family::<DatasetLabels, Gauge>::default();
        registry.register(
            "zfs_dataset_available_bytes",
            "Available space in ZFS dataset in bytes",
            zfs_dataset_available_bytes.clone(),
        );

        let zfs_dataset_referenced_bytes = Family::<DatasetLabels, Gauge>::default();
        registry.register(
            "zfs_dataset_referenced_bytes",
            "Referenced space in ZFS dataset in bytes",
            zfs_dataset_referenced_bytes.clone(),
        );

        let zfs_dataset_mounted = Family::<DatasetLabels, Gauge>::default();
        registry.register(
            "zfs_dataset_mounted",
            "Whether ZFS dataset is mounted (1 = mounted, 0 = not mounted)",
            zfs_dataset_mounted.clone(),
        );

        ZfsListMetrics {
            zfs_dataset_used_bytes,
            zfs_dataset_available_bytes,
            zfs_dataset_referenced_bytes,
            zfs_dataset_mounted,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let dataset_list = get_dataset_list().await?;

        for (_, dataset) in dataset_list.pools {
            self.collect_dataset_metrics(&dataset);
        }

        Ok(())
    }

    fn collect_dataset_metrics(&self, dataset: &Dataset) {
        let labels = DatasetLabels::new(dataset);

        // Used bytes
        if let Some(used) = dataset.properties.used {
            self.zfs_dataset_used_bytes
                .get_or_create(&labels)
                .set(used as i64);
        }

        // Available bytes
        if let Some(available) = dataset.properties.available {
            self.zfs_dataset_available_bytes
                .get_or_create(&labels)
                .set(available as i64);
        }

        // Referenced bytes
        if let Some(referenced) = dataset.properties.referenced {
            self.zfs_dataset_referenced_bytes
                .get_or_create(&labels)
                .set(referenced as i64);
        }

        // Mounted status
        if let Some(mounted) = dataset.properties.mounted {
            self.zfs_dataset_mounted
                .get_or_create(&labels)
                .set(if mounted { 1 } else { 0 });
        }
    }
}

impl DatasetLabels {
    pub fn new(dataset: &Dataset) -> Self {
        DatasetLabels {
            dataset: dataset.name.clone(),
            dataset_type: dataset.dataset_type.to_string().to_lowercase(),
            pool: dataset.pool.clone(),
        }
    }
}
