use crate::zfs::arc::read_arc_stats;
use anyhow::Result;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

pub struct ArcMetrics {
    pub arc_hits: Gauge,
    pub arc_iohits: Gauge,
    pub arc_misses: Gauge,
    pub arc_size_bytes: Gauge,
    pub arc_target_bytes: Gauge,
    pub arc_max_size_bytes: Gauge,
    pub arc_min_size_bytes: Gauge,
    pub arc_data_size_bytes: Gauge,
    pub arc_metadata_size_bytes: Gauge,
    pub arc_overhead_size_bytes: Gauge,
    pub arc_compressed_size_bytes: Gauge,
    pub arc_uncompressed_size_bytes: Gauge,
    pub arc_l2_hits: Gauge,
    pub arc_l2_misses: Gauge,
    pub arc_l2_size: Gauge,
    pub arc_l2_asize: Gauge,
    pub arc_l2_hdr_size: Gauge,
    pub arc_l2_read_bytes_total: Gauge,
    pub arc_l2_write_bytes_total: Gauge,
}

impl ArcMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        let arc_hits = Gauge::default();
        registry.register("arc_hits_total", "ARC hits", arc_hits.clone());

        let arc_iohits = Gauge::default();
        registry.register("arc_iohits_total", "ARC I/O hits", arc_iohits.clone());

        let arc_misses = Gauge::default();
        registry.register("arc_misses_total", "ARC misses", arc_misses.clone());

        let arc_size_bytes = Gauge::default();
        registry.register(
            "arc_size_bytes",
            "Current ARC size in bytes",
            arc_size_bytes.clone(),
        );

        let arc_target_bytes = Gauge::default();
        registry.register(
            "arc_target_bytes",
            "Target ARC size in bytes",
            arc_target_bytes.clone(),
        );

        let arc_max_size_bytes = Gauge::default();
        registry.register(
            "arc_max_size_bytes",
            "Maximum ARC size in bytes",
            arc_max_size_bytes.clone(),
        );

        let arc_min_size_bytes = Gauge::default();
        registry.register(
            "arc_min_size_bytes",
            "Minimum ARC size in bytes",
            arc_min_size_bytes.clone(),
        );

        let arc_data_size_bytes = Gauge::default();
        registry.register(
            "arc_data_size_bytes",
            "ARC data size in bytes",
            arc_data_size_bytes.clone(),
        );

        let arc_metadata_size_bytes = Gauge::default();
        registry.register(
            "arc_metadata_size_bytes",
            "ARC metadata size in bytes",
            arc_metadata_size_bytes.clone(),
        );

        let arc_overhead_size_bytes = Gauge::default();
        registry.register(
            "arc_overhead_size_bytes",
            "ARC overhead size in bytes",
            arc_overhead_size_bytes.clone(),
        );

        let arc_compressed_size_bytes = Gauge::default();
        registry.register(
            "arc_compressed_size_bytes",
            "ARC compressed size in bytes",
            arc_compressed_size_bytes.clone(),
        );

        let arc_uncompressed_size_bytes = Gauge::default();
        registry.register(
            "arc_uncompressed_size_bytes",
            "ARC uncompressed size in bytes",
            arc_uncompressed_size_bytes.clone(),
        );

        let arc_l2_hits = Gauge::default();
        registry.register("arc_l2_hits_total", "L2ARC hits", arc_l2_hits.clone());

        let arc_l2_misses = Gauge::default();
        registry.register("arc_l2_misses_total", "L2ARC misses", arc_l2_misses.clone());

        let arc_l2_size = Gauge::default();
        registry.register(
            "arc_l2_size_bytes",
            "L2ARC size in bytes",
            arc_l2_size.clone(),
        );

        let arc_l2_asize = Gauge::default();
        registry.register(
            "arc_l2_asize_bytes",
            "L2ARC actual size in bytes",
            arc_l2_asize.clone(),
        );

        let arc_l2_hdr_size = Gauge::default();
        registry.register(
            "arc_l2_hdr_size_bytes",
            "L2ARC header size in bytes",
            arc_l2_hdr_size.clone(),
        );

        let arc_l2_read_bytes_total = Gauge::default();
        registry.register(
            "arc_l2_read_bytes_total",
            "Total bytes read from L2ARC",
            arc_l2_read_bytes_total.clone(),
        );

        let arc_l2_write_bytes_total = Gauge::default();
        registry.register(
            "arc_l2_write_bytes_total",
            "Total bytes written to L2ARC",
            arc_l2_write_bytes_total.clone(),
        );

        ArcMetrics {
            arc_hits,
            arc_iohits,
            arc_misses,
            arc_size_bytes,
            arc_target_bytes,
            arc_max_size_bytes,
            arc_min_size_bytes,
            arc_data_size_bytes,
            arc_metadata_size_bytes,
            arc_overhead_size_bytes,
            arc_compressed_size_bytes,
            arc_uncompressed_size_bytes,
            arc_l2_hits,
            arc_l2_misses,
            arc_l2_size,
            arc_l2_asize,
            arc_l2_hdr_size,
            arc_l2_read_bytes_total,
            arc_l2_write_bytes_total,
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let stats = read_arc_stats().await?;

        self.arc_hits.set(stats.hits as i64);
        self.arc_iohits.set(stats.iohits as i64);
        self.arc_misses.set(stats.misses as i64);
        self.arc_size_bytes.set(stats.size_bytes as i64);
        self.arc_target_bytes.set(stats.target_bytes as i64);
        self.arc_max_size_bytes.set(stats.max_size_bytes as i64);
        self.arc_min_size_bytes.set(stats.min_size_bytes as i64);
        self.arc_data_size_bytes.set(stats.data_size_bytes as i64);
        self.arc_metadata_size_bytes
            .set(stats.metadata_size_bytes as i64);
        self.arc_overhead_size_bytes
            .set(stats.overhead_size_bytes as i64);
        self.arc_compressed_size_bytes
            .set(stats.compressed_size_bytes as i64);
        self.arc_uncompressed_size_bytes
            .set(stats.uncompressed_size_bytes as i64);
        self.arc_l2_hits.set(stats.l2_hits as i64);
        self.arc_l2_misses.set(stats.l2_misses as i64);
        self.arc_l2_size.set(stats.l2_size as i64);
        self.arc_l2_asize.set(stats.l2_asize as i64);
        self.arc_l2_hdr_size.set(stats.l2_hdr_size as i64);
        self.arc_l2_read_bytes_total
            .set(stats.l2_read_bytes_total as i64);
        self.arc_l2_write_bytes_total
            .set(stats.l2_write_bytes_total as i64);

        Ok(())
    }
}
