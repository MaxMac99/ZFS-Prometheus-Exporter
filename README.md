# ZFS Prometheus Exporter

A high-performance Prometheus exporter for ZFS metrics, written in Rust. Collects comprehensive
metrics from ZFS pools, datasets, vdevs, ARC, L2ARC, I/O statistics, and scan operations.

## Features

- **Pool Metrics**: Size, allocation, fragmentation, capacity, deduplication ratio, health status,
  state tracking
- **Dataset Metrics**: Space usage (used, available, referenced), mount status
- **VDev Metrics**: Comprehensive metrics for all device types (mirrors, raidz, special devices,
  L2ARC, logs, spares)
    - Space allocation and health
    - Read/write/checksum errors
    - Scan progress and self-healing statistics
    - Trim status and operations
- **Scan Metrics**: Scrub and resilver operations with detailed state tracking
- **ARC Metrics**: Cache hits/misses, size management, L2ARC statistics
- **I/O Statistics**:
    - **Size Distribution**: Operations by request size (sync/async read/write, scrub, trim,
      rebuild)
    - **Latency Distribution**: Wait time histograms across all queue types
    - **Queue Depths**: Pending and active operations for sync/async/scrub/trim/rebuild queues
- **Async Collection**: Concurrent metric gathering for optimal performance

## Requirements

- **ZFS**: OpenZFS with JSON output support (`-j` and `--json-int` flags)
- **Linux**: `/proc/spl/kstat/zfs/arcstats` available
- **Permissions**: Root privileges to run `zpool` and `zfs` commands
- **NixOS**: Recommended for deployment

## Installation

### NixOS with Flakes

Add to your `flake.nix` inputs:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    zfs-exporter = {
      url = "github:MaxMac99/zfs-prometheus-exporter";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, zfs-exporter, ... }: {
    nixosConfigurations.your-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        zfs-exporter.nixosModules.default
        {
          # Enable ZFS
          boot.supportedFilesystems = [ "zfs" ];

          # Enable exporter
          services.zfs-prometheus-exporter = {
            enable = true;
            port = 9134;
            openFirewall = true;
          };
        }
      ];
    };
  };
}
```

Rebuild your system:

```bash
sudo nixos-rebuild switch --flake .#your-host
```

### Configuration Options

All available options for `services.zfs-prometheus-exporter`:

| Option            | Type       | Default     | Description                         |
|-------------------|------------|-------------|-------------------------------------|
| `enable`          | bool       | `false`     | Enable the ZFS exporter service     |
| `package`         | package    | auto        | Package to use                      |
| `port`            | port       | `9134`      | Port to listen on                   |
| `host`            | string     | `"0.0.0.0"` | Host address to bind to             |
| `openFirewall`    | bool       | `false`     | Open firewall for the exporter port |
| `extraFlags`      | list       | `[]`        | Extra command-line flags            |
| `environmentFile` | path\|null | `null`      | Environment file for configuration  |

### Advanced NixOS Configuration

#### With Prometheus and Grafana

```nix
{ config, pkgs, ... }:

{
  boot.supportedFilesystems = [ "zfs" ];

  # ZFS Exporter
  services.zfs-prometheus-exporter = {
    enable = true;
    openFirewall = true;
  };

  # Prometheus
  services.prometheus = {
    enable = true;
    port = 9090;

    scrapeConfigs = [{
      job_name = "zfs";
      scrape_interval = "30s";
      static_configs = [{
        targets = [ "localhost:9134" ];
        labels = {
          instance = config.networking.hostName;
        };
      }];
    }];

    ruleFiles = [
      (pkgs.writeText "zfs-alerts.yml" ''
        groups:
          - name: zfs_alerts
            rules:
              - alert: ZFSPoolHighCapacity
                expr: zpool_capacity_percent > 80
                for: 5m
                labels:
                  severity: warning
                annotations:
                  summary: "ZFS pool {{ $labels.pool }} at {{ $value }}% capacity"

              - alert: ZFSPoolErrors
                expr: vdev_read_errors_total + vdev_write_errors_total + vdev_checksum_errors_total > 0
                for: 1m
                labels:
                  severity: critical
                annotations:
                  summary: "ZFS errors detected on {{ $labels.pool }}"
      '')
    ];
  };

  # Grafana
  services.grafana = {
    enable = true;
    settings.server = {
      http_port = 3000;
      http_addr = "0.0.0.0";
    };
  };
}
```

#### Behind Nginx Reverse Proxy

```nix
{ config, pkgs, ... }:

{
  services.zfs-prometheus-exporter = {
    enable = true;
    host = "127.0.0.1";
  };

  services.nginx = {
    enable = true;
    virtualHosts."metrics.example.com" = {
      enableACME = true;
      forceSSL = true;

      locations."/zfs/metrics" = {
        proxyPass = "http://127.0.0.1:9134/metrics";
        extraConfig = ''
          auth_basic "Metrics";
          auth_basic_user_file /etc/nginx/metrics.htpasswd;
        '';
      };
    };
  };
}
```

## Development

### Using Nix Development Shell

```bash
# Enter development environment
nix develop

# Build
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Auto-rebuild on changes
cargo watch -x run
```

### Building the Package

```bash
# Build with Nix
nix build

# Run the built binary
sudo ./result/bin/zfs-prometheus-exporter

# Build and run directly
nix run
```

## Usage

The exporter runs as a systemd service when configured via NixOS module. It exposes metrics on the
configured port (default: 9134).

### Endpoints

- `/metrics` - Prometheus metrics
- `/health` - Health check endpoint

### Command Line Options

- `--port, -p`: Port to listen on (default: 9134)
- `--host, -H`: Host to bind to (default: 0.0.0.0)

## Metrics Reference

### Pool State Metrics

| Metric                        | Type  | Description                               | Labels           |
|-------------------------------|-------|-------------------------------------------|------------------|
| `zpool_state`                 | Gauge | Pool state (1 = current state, 0 = other) | `pool`, `state`  |
| `zpool_size_bytes`            | Gauge | Total pool size                           | `pool`           |
| `zpool_allocated_bytes`       | Gauge | Allocated space                           | `pool`           |
| `zpool_free_bytes`            | Gauge | Free space                                | `pool`           |
| `zpool_checkpoint_bytes`      | Gauge | Checkpoint space                          | `pool`           |
| `zpool_expandsize_bytes`      | Gauge | Expandable size                           | `pool`           |
| `zpool_fragmentation_percent` | Gauge | Fragmentation percentage                  | `pool`           |
| `zpool_capacity_percent`      | Gauge | Capacity percentage                       | `pool`           |
| `zpool_dedupratio`            | Gauge | Deduplication ratio                       | `pool`           |
| `zpool_health`                | Gauge | Health status (1 = current, 0 = other)    | `pool`, `health` |

### Scan Metrics

| Metric                                  | Type  | Description                         | Labels                      |
|-----------------------------------------|-------|-------------------------------------|-----------------------------|
| `zpool_scan_state`                      | Gauge | Scan state (1 = current, 0 = other) | `pool`, `function`, `state` |
| `zpool_scan_start_time`                 | Gauge | Scan start timestamp                | `pool`, `function`          |
| `zpool_scan_end_time`                   | Gauge | Scan end timestamp (0 if running)   | `pool`, `function`          |
| `zpool_scan_to_examine_bytes`           | Gauge | Total bytes to examine              | `pool`, `function`          |
| `zpool_scan_examined_bytes`             | Gauge | Bytes examined                      | `pool`, `function`          |
| `zpool_scan_skipped_bytes`              | Gauge | Bytes skipped                       | `pool`, `function`          |
| `zpool_scan_processed_bytes`            | Gauge | Bytes processed                     | `pool`, `function`          |
| `zpool_scan_errors_total`               | Gauge | Scan errors                         | `pool`, `function`          |
| `zpool_scan_bytes_per_scan`             | Gauge | Bytes per scan operation            | `pool`, `function`          |
| `zpool_scan_pass_start`                 | Gauge | Scan pass start timestamp           | `pool`, `function`          |
| `zpool_scan_scrub_pause`                | Gauge | Scrub pause count                   | `pool`, `function`          |
| `zpool_scan_scrub_spent_paused_seconds` | Gauge | Time spent paused                   | `pool`, `function`          |
| `zpool_scan_issued_bytes_per_scan`      | Gauge | Bytes issued per scan               | `pool`, `function`          |
| `zpool_scan_issued_bytes`               | Gauge | Total bytes issued                  | `pool`, `function`          |

### VDev Metrics

| Metric                         | Type  | Description                         | Labels                                                                      |
|--------------------------------|-------|-------------------------------------|-----------------------------------------------------------------------------|
| `vdev_state`                   | Gauge | VDev state (1 = current, 0 = other) | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`, `state`      |
| `vdev_alloc_space_bytes`       | Gauge | Allocated space                     | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_total_space_bytes`       | Gauge | Total space                         | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_def_space_bytes`         | Gauge | Deferred space                      | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_phys_space_bytes`        | Gauge | Physical space                      | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_rep_dev_size_bytes`      | Gauge | Replaceable device size             | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_ex_dev_size_bytes`       | Gauge | Expandable device size              | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_read_errors_total`       | Gauge | Read errors                         | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_write_errors_total`      | Gauge | Write errors                        | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_checksum_errors_total`   | Gauge | Checksum errors                     | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_self_healed_bytes`       | Gauge | Self-healed bytes                   | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_scan_processed_bytes`    | Gauge | Scan processed bytes                | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_checkpoint_space_bytes`  | Gauge | Checkpoint space                    | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_resilver_deferred_bytes` | Gauge | Resilver deferred bytes             | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_slow_ios_total`          | Gauge | Slow I/O operations                 | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_trim_state`              | Gauge | Trim state (1 = current, 0 = other) | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`, `trim_state` |
| `vdev_trimmed_bytes`           | Gauge | Trimmed bytes                       | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_to_trim_bytes`           | Gauge | Bytes to trim                       | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_trim_time_seconds`       | Gauge | Trim time                           | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_trim_errors_total`       | Gauge | Trim errors                         | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |
| `vdev_trim_notsup_total`       | Gauge | Trim operations not supported       | `pool`, `category`, `vdev`, `vdev_type`, `vdev_class`, `path`               |

**VDev Categories**: `root`, `dedup`, `special`, `logs`, `l2cache`, `spares`

### Dataset Metrics

| Metric                         | Type  | Description                         | Labels                            |
|--------------------------------|-------|-------------------------------------|-----------------------------------|
| `zfs_dataset_used_bytes`       | Gauge | Used space                          | `dataset`, `dataset_type`, `pool` |
| `zfs_dataset_available_bytes`  | Gauge | Available space                     | `dataset`, `dataset_type`, `pool` |
| `zfs_dataset_referenced_bytes` | Gauge | Referenced space                    | `dataset`, `dataset_type`, `pool` |
| `zfs_dataset_mounted`          | Gauge | Mount status (1 = mounted, 0 = not) | `dataset`, `dataset_type`, `pool` |

### I/O Size Distribution

| Metric                           | Type  | Description               | Labels                                     |
|----------------------------------|-------|---------------------------|--------------------------------------------|
| `zpool_io_size_operations_total` | Gauge | Operations by size bucket | `pool`, `req_size`, `operation`, `io_type` |

**Operations**: `sync_read`, `sync_write`, `async_read`, `async_write`, `scrub_read`, `trim_write`,
`rebuild_write`
**I/O Types**: `independent`, `aggregated`

### I/O Latency Distribution

| Metric                           | Type  | Description                  | Labels                            |
|----------------------------------|-------|------------------------------|-----------------------------------|
| `zpool_latency_operations_total` | Gauge | Operations by latency bucket | `pool`, `latency_ns`, `operation` |

**Operations**: `total_wait_read`, `total_wait_write`, `disk_wait_read`, `disk_wait_write`,
`syncq_wait_read`, `syncq_wait_write`, `asyncq_wait_read`, `asyncq_wait_write`, `scrub`, `trim`,
`rebuild`

### Queue Depth Metrics

| Metric                         | Type  | Description            | Labels                                     |
|--------------------------------|-------|------------------------|--------------------------------------------|
| `zpool_queue_capacity_bytes`   | Gauge | Pool capacity          | `pool`                                     |
| `zpool_queue_operations_total` | Gauge | Operation counts       | `pool`, `operation`                        |
| `zpool_queue_bandwidth_bytes`  | Gauge | Bandwidth by operation | `pool`, `operation`                        |
| `zpool_queue_depth`            | Gauge | Queue depth            | `pool`, `queue_type`, `operation`, `state` |

**Queue Types**: `sync`, `async`, `scrub`, `trim`, `rebuild`
**States**: `pending`, `active`

### ARC Metrics

| Metric                        | Type  | Description                |
|-------------------------------|-------|----------------------------|
| `arc_hits_total`              | Gauge | ARC cache hits             |
| `arc_iohits_total`            | Gauge | ARC I/O hits               |
| `arc_misses_total`            | Gauge | ARC cache misses           |
| `arc_size_bytes`              | Gauge | Current ARC size           |
| `arc_target_bytes`            | Gauge | Target ARC size            |
| `arc_max_size_bytes`          | Gauge | Maximum ARC size           |
| `arc_min_size_bytes`          | Gauge | Minimum ARC size           |
| `arc_data_size_bytes`         | Gauge | ARC data size              |
| `arc_metadata_size_bytes`     | Gauge | ARC metadata size          |
| `arc_overhead_size_bytes`     | Gauge | ARC overhead size          |
| `arc_compressed_size_bytes`   | Gauge | ARC compressed data size   |
| `arc_uncompressed_size_bytes` | Gauge | ARC uncompressed data size |

### L2ARC Metrics

| Metric                     | Type  | Description                  |
|----------------------------|-------|------------------------------|
| `arc_l2_hits_total`        | Gauge | L2ARC cache hits             |
| `arc_l2_misses_total`      | Gauge | L2ARC cache misses           |
| `arc_l2_size_bytes`        | Gauge | L2ARC size                   |
| `arc_l2_asize_bytes`       | Gauge | L2ARC actual size            |
| `arc_l2_hdr_size_bytes`    | Gauge | L2ARC header size            |
| `arc_l2_read_bytes_total`  | Gauge | Total bytes read from L2ARC  |
| `arc_l2_write_bytes_total` | Gauge | Total bytes written to L2ARC |

## Example PromQL Queries

### Pool Health

```promql
# Pool capacity percentage
zpool_capacity_percent

# Pool fragmentation
zpool_fragmentation_percent

# Free space in GB
zpool_free_bytes / 1024 / 1024 / 1024

# Unhealthy pools
zpool_health{health!="online"} == 1
```

### Performance Monitoring

```promql
# ARC hit rate percentage
100 * arc_hits_total / (arc_hits_total + arc_misses_total)

# L2ARC hit rate percentage
100 * arc_l2_hits_total / (arc_l2_hits_total + arc_l2_misses_total)

# ARC efficiency
rate(arc_hits_total[5m])

# ARC memory pressure (current vs target)
arc_size_bytes / arc_target_bytes
```

### Disk Health

```promql
# Total errors per vdev
sum by (pool, vdev, path) (
  vdev_read_errors_total +
  vdev_write_errors_total +
  vdev_checksum_errors_total
)

# VDevs with any errors
(vdev_read_errors_total + vdev_write_errors_total + vdev_checksum_errors_total) > 0

# Slow I/O operations
vdev_slow_ios_total > 0
```

### Scan Progress

```promql
# Scan completion percentage
100 * zpool_scan_examined_bytes / zpool_scan_to_examine_bytes

# Scan rate in MB/s
rate(zpool_scan_examined_bytes[5m]) / 1024 / 1024

# Estimated time remaining (seconds)
(zpool_scan_to_examine_bytes - zpool_scan_examined_bytes) /
rate(zpool_scan_examined_bytes[5m])

# Active scans
zpool_scan_state{state="scanning"} == 1
```

### I/O Analysis

```promql
# Queue depths by type
zpool_queue_depth{state="pending"}

# Operations per second by type
rate(zpool_queue_operations_total[5m])

# Bandwidth by operation (MB/s)
rate(zpool_queue_bandwidth_bytes[5m]) / 1024 / 1024

# Hot latency buckets
topk(10, zpool_latency_operations_total)
```

## Troubleshooting

### Service Not Starting

Check the service status:

```bash
sudo systemctl status zfs-prometheus-exporter
sudo journalctl -u zfs-prometheus-exporter -n 50 --no-pager
```

Common issues:

1. **ZFS not enabled in configuration**
   ```nix
   boot.supportedFilesystems = [ "zfs" ];
   ```

2. **Port already in use**
   ```bash
   sudo netstat -tlnp | grep 9134
   ```

3. **Firewall blocking access**
   ```nix
   services.zfs-prometheus-exporter.openFirewall = true;
   ```

### No Metrics Appearing

Check the endpoint:

```bash
curl http://localhost:9134/metrics
curl http://localhost:9134/health
```

Verify ZFS is working:

```bash
sudo zpool list -j --json-int
sudo zfs list -j --json-int
cat /proc/spl/kstat/zfs/arcstats
```

### Performance Tuning

The exporter uses async I/O and runs metric collection concurrently for optimal performance:

- Metrics collected on-demand when Prometheus scrapes
- All ZFS commands run in parallel using `tokio::try_join!`
- Typical memory usage: < 50MB
- CPU usage: negligible except during scrapes

For high-frequency scraping, consider adjusting:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'zfs'
    scrape_interval: 30s  # Adjust based on pool size
    scrape_timeout: 10s
```

## License

Apache-2.0 - see LICENSE file for details

## Resources

- [OpenZFS Documentation](https://openzfs.github.io/openzfs-docs/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [NixOS Manual](https://nixos.org/manual/nixos/stable/)
- [ZFS on NixOS](https://nixos.wiki/wiki/ZFS)

## Credits

Built with:

- [Rust](https://www.rust-lang.org/)
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [prometheus-client](https://github.com/prometheus/client_rust) - Prometheus client
- [Tokio](https://tokio.rs/) - Async runtime
- [Nix](https://nixos.org/) - Reproducible builds and deployment