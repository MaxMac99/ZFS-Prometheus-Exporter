use crate::zfs::zpool_status::{
    get_pool_status, PoolState, PoolStatus, ScanInfo, ScanState, TrimState, VDev,
};
use anyhow::Result;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use strum::IntoEnumIterator;

pub struct ZpoolStatusMetrics {
    // Pool Status
    pub zpool_state: Family<ZpoolStateLabels, Gauge>,

    // Scan Stats
    pub zpool_scan_state: Family<ZpoolScanStateLabels, Gauge>,
    pub zpool_scan_start_time: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_end_time: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_to_examine: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_examined: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_skipped: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_processed: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_errors: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_bytes_per_scan: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_pass_start: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_scrub_pause: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_scrub_spent_paused: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_issued_bytes_per_scan: Family<ZpoolScanLabels, Gauge>,
    pub zpool_scan_issued: Family<ZpoolScanLabels, Gauge>,

    // VDev State
    pub vdev_state: Family<VDevStateLabels, Gauge>,
    pub vdev_alloc_space: Family<VDevLabels, Gauge>,
    pub vdev_total_space: Family<VDevLabels, Gauge>,
    pub vdev_def_space: Family<VDevLabels, Gauge>,
    pub vdev_phys_space: Family<VDevLabels, Gauge>,
    pub vdev_rep_dev_size: Family<VDevLabels, Gauge>,
    pub vdev_ex_dev_size: Family<VDevLabels, Gauge>,
    pub vdev_read_errors: Family<VDevLabels, Gauge>,
    pub vdev_write_errors: Family<VDevLabels, Gauge>,
    pub vdev_checksum_errors: Family<VDevLabels, Gauge>,

    // VDev Health
    pub vdev_self_healed: Family<VDevLabels, Gauge>,
    pub vdev_scan_processed: Family<VDevLabels, Gauge>,
    pub vdev_checkpoint_space: Family<VDevLabels, Gauge>,
    pub vdev_resilver_deferred: Family<VDevLabels, Gauge>,
    pub vdev_slow_ios: Family<VDevLabels, Gauge>,

    // VDev Trim
    pub vdev_trim_state: Family<VDevTrimStateLabels, Gauge>,
    pub vdev_trimmed: Family<VDevLabels, Gauge>,
    pub vdev_to_trim: Family<VDevLabels, Gauge>,
    pub vdev_trim_time: Family<VDevLabels, Gauge>,
    pub vdev_trim_errors: Family<VDevLabels, Gauge>,
    pub vdev_trim_notsup: Family<VDevLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolStateLabels {
    pub pool: String,
    pub state: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolScanStateLabels {
    pub pool: String,
    pub function: String,
    pub state: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ZpoolScanLabels {
    pub pool: String,
    pub function: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VDevLabels {
    pub pool: String,
    pub category: String,
    pub vdev: String,
    pub vdev_type: String,
    pub vdev_class: String,
    pub path: String,
    pub parent: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VDevStateLabels {
    pub pool: String,
    pub category: String,
    pub vdev: String,
    pub vdev_type: String,
    pub vdev_class: String,
    pub path: String,
    pub parent: String,
    pub state: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VDevTrimStateLabels {
    pub pool: String,
    pub category: String,
    pub vdev: String,
    pub vdev_type: String,
    pub vdev_class: String,
    pub path: String,
    pub parent: String,
    pub trim_state: String,
}

impl ZpoolStatusMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        // Pool state metrics
        let zpool_state = Family::<ZpoolStateLabels, Gauge>::default();
        registry.register(
            "zpool_state",
            "ZFS pool state (1 = current state, 0 = other states)",
            zpool_state.clone(),
        );

        // Scan state metrics
        let zpool_scan_state = Family::<ZpoolScanStateLabels, Gauge>::default();
        registry.register(
            "zpool_scan_state",
            "ZFS pool scan/scrub state (1 = current state, 0 = other states)",
            zpool_scan_state.clone(),
        );

        let zpool_scan_start_time = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_start_time",
            "Unix timestamp when the scan started",
            zpool_scan_start_time.clone(),
        );

        let zpool_scan_end_time = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_end_time",
            "Unix timestamp when the scan ended (0 if still running)",
            zpool_scan_end_time.clone(),
        );

        let zpool_scan_to_examine = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_to_examine_bytes",
            "Total bytes to examine during scan",
            zpool_scan_to_examine.clone(),
        );

        let zpool_scan_examined = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_examined_bytes",
            "Bytes examined so far during scan",
            zpool_scan_examined.clone(),
        );

        let zpool_scan_skipped = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_skipped_bytes",
            "Bytes skipped during scan",
            zpool_scan_skipped.clone(),
        );

        let zpool_scan_processed = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_processed_bytes",
            "Bytes processed during scan",
            zpool_scan_processed.clone(),
        );

        let zpool_scan_errors = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_errors_total",
            "Number of errors encountered during scan",
            zpool_scan_errors.clone(),
        );

        let zpool_scan_bytes_per_scan = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_bytes_per_scan",
            "Bytes scanned per scan operation",
            zpool_scan_bytes_per_scan.clone(),
        );

        let zpool_scan_pass_start = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_pass_start",
            "Scan pass start timestamp",
            zpool_scan_pass_start.clone(),
        );

        let zpool_scan_scrub_pause = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_scrub_pause",
            "Scrub pause count",
            zpool_scan_scrub_pause.clone(),
        );

        let zpool_scan_scrub_spent_paused = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_scrub_spent_paused_seconds",
            "Time spent paused during scrub",
            zpool_scan_scrub_spent_paused.clone(),
        );

        let zpool_scan_issued_bytes_per_scan = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_issued_bytes_per_scan",
            "Bytes issued per scan operation",
            zpool_scan_issued_bytes_per_scan.clone(),
        );

        let zpool_scan_issued = Family::<ZpoolScanLabels, Gauge>::default();
        registry.register(
            "zpool_scan_issued_bytes",
            "Total bytes issued during scan",
            zpool_scan_issued.clone(),
        );

        // VDev state metrics
        let vdev_state = Family::<VDevStateLabels, Gauge>::default();
        registry.register(
            "vdev_state",
            "VDev state (1 = current state, 0 = other states)",
            vdev_state.clone(),
        );

        // VDev space metrics
        let vdev_alloc_space = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_alloc_space_bytes",
            "Allocated space on VDev",
            vdev_alloc_space.clone(),
        );

        let vdev_total_space = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_total_space_bytes",
            "Total space on VDev",
            vdev_total_space.clone(),
        );

        let vdev_def_space = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_def_space_bytes",
            "Deferred space on VDev",
            vdev_def_space.clone(),
        );

        let vdev_phys_space = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_phys_space_bytes",
            "Physical space on VDev",
            vdev_phys_space.clone(),
        );

        let vdev_rep_dev_size = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_rep_dev_size_bytes",
            "Replaceable device size",
            vdev_rep_dev_size.clone(),
        );

        let vdev_ex_dev_size = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_ex_dev_size_bytes",
            "Expandable device size",
            vdev_ex_dev_size.clone(),
        );

        // VDev error metrics
        let vdev_read_errors = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_read_errors_total",
            "Number of read errors on VDev",
            vdev_read_errors.clone(),
        );

        let vdev_write_errors = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_write_errors_total",
            "Number of write errors on VDev",
            vdev_write_errors.clone(),
        );

        let vdev_checksum_errors = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_checksum_errors_total",
            "Number of checksum errors on VDev",
            vdev_checksum_errors.clone(),
        );

        // VDev health metrics
        let vdev_self_healed = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_self_healed_bytes",
            "Bytes self-healed on VDev",
            vdev_self_healed.clone(),
        );

        let vdev_scan_processed = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_scan_processed_bytes",
            "Bytes processed during scan on VDev",
            vdev_scan_processed.clone(),
        );

        let vdev_checkpoint_space = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_checkpoint_space_bytes",
            "Checkpoint space on VDev",
            vdev_checkpoint_space.clone(),
        );

        let vdev_resilver_deferred = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_resilver_deferred_bytes",
            "Deferred resilver bytes on VDev",
            vdev_resilver_deferred.clone(),
        );

        let vdev_slow_ios = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_slow_ios_total",
            "Number of slow I/O operations on VDev",
            vdev_slow_ios.clone(),
        );

        // VDev trim metrics
        let vdev_trim_state = Family::<VDevTrimStateLabels, Gauge>::default();
        registry.register(
            "vdev_trim_state",
            "VDev trim state (1 = current state, 0 = other states)",
            vdev_trim_state.clone(),
        );

        let vdev_trimmed = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_trimmed_bytes",
            "Bytes trimmed on VDev",
            vdev_trimmed.clone(),
        );

        let vdev_to_trim = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_to_trim_bytes",
            "Bytes to trim on VDev",
            vdev_to_trim.clone(),
        );

        let vdev_trim_time = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_trim_time_seconds",
            "Time spent trimming VDev",
            vdev_trim_time.clone(),
        );

        let vdev_trim_errors = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_trim_errors_total",
            "Number of trim errors on VDev",
            vdev_trim_errors.clone(),
        );

        let vdev_trim_notsup = Family::<VDevLabels, Gauge>::default();
        registry.register(
            "vdev_trim_notsup_total",
            "Number of trim operations not supported on VDev",
            vdev_trim_notsup.clone(),
        );

        ZpoolStatusMetrics {
            zpool_state,
            zpool_scan_state,
            zpool_scan_start_time,
            zpool_scan_end_time,
            zpool_scan_to_examine,
            zpool_scan_examined,
            zpool_scan_skipped,
            zpool_scan_processed,
            zpool_scan_errors,
            zpool_scan_bytes_per_scan,
            zpool_scan_pass_start,
            zpool_scan_scrub_pause,
            zpool_scan_scrub_spent_paused,
            zpool_scan_issued_bytes_per_scan,
            zpool_scan_issued,
            vdev_state,
            vdev_alloc_space,
            vdev_total_space,
            vdev_def_space,
            vdev_phys_space,
            vdev_rep_dev_size,
            vdev_ex_dev_size,
            vdev_read_errors,
            vdev_write_errors,
            vdev_checksum_errors,
            vdev_self_healed,
            vdev_scan_processed,
            vdev_checkpoint_space,
            vdev_resilver_deferred,
            vdev_slow_ios,
            vdev_trim_state,
            vdev_trimmed,
            vdev_to_trim,
            vdev_trim_time,
            vdev_trim_errors,
            vdev_trim_notsup,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let status = get_pool_status().await?;

        for (_, pool) in status.pools {
            self.collect_zpool_state(&pool);
            self.collect_zpool_scan_stats(&pool);

            for vdev in pool.vdevs.values() {
                self.collect_zpool_vdev_metrics(&pool, "root", vdev, None);
            }
            for vdev in pool.dedup.values() {
                self.collect_zpool_vdev_metrics(&pool, "dedup", vdev, None);
            }
            for vdev in pool.special.values() {
                self.collect_zpool_vdev_metrics(&pool, "special", vdev, None);
            }
            for vdev in pool.logs.values() {
                self.collect_zpool_vdev_metrics(&pool, "logs", vdev, None);
            }
            for vdev in pool.l2cache.values() {
                self.collect_zpool_vdev_metrics(&pool, "l2cache", vdev, None);
            }
            for vdev in pool.spares.values() {
                self.collect_zpool_vdev_metrics(&pool, "spares", vdev, None);
            }
        }

        Ok(())
    }

    fn collect_zpool_state(&self, pool: &PoolStatus) {
        PoolState::iter().for_each(|state| {
            let labels = ZpoolStateLabels::new(pool, state);
            let gauge = self.zpool_state.get_or_create(&labels);
            if pool.state == state {
                gauge.set(1);
            } else {
                gauge.set(0);
            }
        });
    }

    fn collect_zpool_scan_stats(&self, pool: &PoolStatus) {
        let scan = match &pool.scan {
            Some(scan) => scan,
            None => return,
        };
        ScanState::iter().for_each(|state| {
            let labels = ZpoolScanStateLabels::new(pool, scan, state);
            let gauge = self.zpool_scan_state.get_or_create(&labels);
            if scan.state == state {
                gauge.set(1);
            } else {
                gauge.set(0);
            }
        });

        let labels = ZpoolScanLabels::new(pool, scan);
        self.zpool_scan_start_time
            .get_or_create(&labels)
            .set(scan.start_time.timestamp());
        self.zpool_scan_end_time
            .get_or_create(&labels)
            .set(scan.end_time.map_or(0, |t| t.timestamp()));
        self.zpool_scan_to_examine
            .get_or_create(&labels)
            .set(scan.to_examine as i64);
        self.zpool_scan_examined
            .get_or_create(&labels)
            .set(scan.examined as i64);
        self.zpool_scan_skipped
            .get_or_create(&labels)
            .set(scan.skipped as i64);
        self.zpool_scan_processed
            .get_or_create(&labels)
            .set(scan.processed as i64);
        self.zpool_scan_errors
            .get_or_create(&labels)
            .set(scan.errors as i64);
        self.zpool_scan_bytes_per_scan
            .get_or_create(&labels)
            .set(scan.bytes_per_scan as i64);
        self.zpool_scan_pass_start
            .get_or_create(&labels)
            .set(scan.pass_start as i64);
        self.zpool_scan_scrub_pause
            .get_or_create(&labels)
            .set(scan.scrub_pause as i64);
        self.zpool_scan_scrub_spent_paused
            .get_or_create(&labels)
            .set(scan.scrub_spent_paused as i64);
        self.zpool_scan_issued_bytes_per_scan
            .get_or_create(&labels)
            .set(scan.issued_bytes_per_scan as i64);
        self.zpool_scan_issued
            .get_or_create(&labels)
            .set(scan.issued as i64);
    }

    fn collect_zpool_vdev_metrics(
        &self,
        pool: &PoolStatus,
        category: &str,
        vdev: &VDev,
        parent: Option<&VDev>,
    ) {
        PoolState::iter().for_each(|state| {
            let labels = VDevStateLabels::new(
                pool,
                category,
                vdev,
                parent.map(|p| p.name.clone()),
                &state.to_string(),
            );
            let gauge = self.vdev_state.get_or_create(&labels);
            if vdev.state == state {
                gauge.set(1);
            } else {
                gauge.set(0);
            }
        });

        let labels = VDevLabels::new(pool, category, vdev, parent.map(|p| p.name.clone()));

        // Space metrics
        if let Some(value) = vdev.alloc_space {
            self.vdev_alloc_space
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.total_space {
            self.vdev_total_space
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.def_space {
            self.vdev_def_space.get_or_create(&labels).set(value as i64);
        }
        if let Some(value) = vdev.phys_space {
            self.vdev_phys_space
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.rep_dev_size {
            self.vdev_rep_dev_size
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.ex_dev_size {
            self.vdev_ex_dev_size
                .get_or_create(&labels)
                .set(value as i64);
        }

        // Error metrics
        if let Some(value) = vdev.read_errors {
            self.vdev_read_errors
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.write_errors {
            self.vdev_write_errors
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.checksum_errors {
            self.vdev_checksum_errors
                .get_or_create(&labels)
                .set(value as i64);
        }

        // Health metrics
        if let Some(value) = vdev.self_healed {
            self.vdev_self_healed
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.scan_processed {
            self.vdev_scan_processed
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.checkpoint_space {
            self.vdev_checkpoint_space
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.resilver_deferred {
            self.vdev_resilver_deferred
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.slow_ios {
            self.vdev_slow_ios.get_or_create(&labels).set(value as i64);
        }

        // Trim state metric
        if let Some(trim_state) = vdev.trim_state {
            TrimState::iter().for_each(|state| {
                let labels = VDevTrimStateLabels::new(
                    pool,
                    category,
                    vdev,
                    parent.map(|p| p.name.clone()),
                    &state.to_string(),
                );
                let gauge = self.vdev_trim_state.get_or_create(&labels);
                if trim_state == state {
                    gauge.set(1);
                } else {
                    gauge.set(0);
                }
            });
        }

        // Trim metrics
        if let Some(value) = vdev.trimmed {
            self.vdev_trimmed.get_or_create(&labels).set(value as i64);
        }
        if let Some(value) = vdev.to_trim {
            self.vdev_to_trim.get_or_create(&labels).set(value as i64);
        }
        if let Some(value) = vdev.trim_time {
            self.vdev_trim_time.get_or_create(&labels).set(value as i64);
        }
        if let Some(value) = vdev.trim_errors {
            self.vdev_trim_errors
                .get_or_create(&labels)
                .set(value as i64);
        }
        if let Some(value) = vdev.trim_notsup {
            self.vdev_trim_notsup
                .get_or_create(&labels)
                .set(value as i64);
        }

        for child_vdev in vdev.children.values() {
            self.collect_zpool_vdev_metrics(pool, category, child_vdev, Some(vdev));
        }
    }
}

impl ZpoolStateLabels {
    pub fn new(pool: &PoolStatus, state: PoolState) -> Self {
        ZpoolStateLabels {
            pool: pool.name.clone(),
            state: state.to_string().to_lowercase(),
        }
    }
}

impl ZpoolScanLabels {
    pub fn new(pool: &PoolStatus, scan: &ScanInfo) -> Self {
        ZpoolScanLabels {
            pool: pool.name.clone(),
            function: scan.function.to_string().to_lowercase(),
        }
    }
}

impl ZpoolScanStateLabels {
    pub fn new(pool: &PoolStatus, scan: &ScanInfo, state: ScanState) -> Self {
        ZpoolScanStateLabels {
            pool: pool.name.clone(),
            function: scan.function.to_string().to_lowercase(),
            state: state.to_string().to_lowercase(),
        }
    }
}

impl VDevLabels {
    pub fn new(pool: &PoolStatus, category: &str, vdev: &VDev, parent: Option<String>) -> Self {
        VDevLabels {
            pool: pool.name.clone(),
            category: category.to_string(),
            vdev: vdev.name.to_string(),
            vdev_type: vdev.vdev_type.clone(),
            vdev_class: vdev.class.to_string(),
            path: vdev.path.clone().unwrap_or_default(),
            parent: parent.unwrap_or_default(),
        }
    }
}

impl VDevStateLabels {
    pub fn new(
        pool: &PoolStatus,
        category: &str,
        vdev: &VDev,
        parent: Option<String>,
        state: &str,
    ) -> Self {
        let labels = VDevLabels::new(pool, category, vdev, parent);
        VDevStateLabels {
            pool: labels.pool,
            category: labels.category,
            vdev: labels.vdev,
            vdev_type: labels.vdev_type,
            vdev_class: labels.vdev_class,
            path: labels.path,
            parent: labels.parent,
            state: state.to_string().to_lowercase(),
        }
    }
}

impl VDevTrimStateLabels {
    pub fn new(
        pool: &PoolStatus,
        category: &str,
        vdev: &VDev,
        parent: Option<String>,
        trim_state: &str,
    ) -> Self {
        let labels = VDevLabels::new(pool, category, vdev, parent);
        VDevTrimStateLabels {
            pool: labels.pool,
            category: labels.category,
            vdev: labels.vdev,
            vdev_type: labels.vdev_type,
            vdev_class: labels.vdev_class,
            path: labels.path,
            parent: labels.parent,
            trim_state: trim_state.to_string().to_lowercase(),
        }
    }
}
