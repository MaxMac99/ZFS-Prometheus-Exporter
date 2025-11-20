use anyhow::Context;
use chrono::{DateTime, Utc};
use enum_display::EnumDisplay;
use serde::Deserialize;
use std::collections::HashMap;
use strum::EnumIter;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct ZpoolStatus {
    #[serde(default)]
    pub pools: HashMap<String, PoolStatus>,
}

#[derive(Debug, Deserialize)]
pub struct PoolStatus {
    pub name: String,
    pub state: PoolState,
    #[serde(default)]
    #[allow(dead_code)]
    pub status: String,
    #[serde(default, rename = "scan_stats")]
    pub scan: Option<ScanInfo>,
    #[serde(default, rename = "removal_stats")]
    #[allow(dead_code)]
    pub removal: Option<RemovalInfo>,
    #[serde(default, rename = "checkpoint_stats")]
    #[allow(dead_code)]
    pub checkpoint: Option<CheckpointInfo>,
    #[serde(default)]
    pub vdevs: HashMap<String, VDev>,
    #[serde(default)]
    pub dedup: HashMap<String, VDev>,
    #[serde(default)]
    pub special: HashMap<String, VDev>,
    #[serde(default)]
    pub logs: HashMap<String, VDev>,
    #[serde(default)]
    pub l2cache: HashMap<String, VDev>,
    #[serde(default)]
    pub spares: HashMap<String, VDev>,
}

#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, EnumIter, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PoolState {
    Online,
    Degraded,
    Faulted,
    Offline,
    Unavailable,
    Removed,
    Suspended,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct ScanInfo {
    pub function: ScanFunction,
    pub state: ScanState,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    #[serde(default, with = "chrono::serde::ts_seconds_option")]
    pub end_time: Option<DateTime<Utc>>,
    pub to_examine: u64,
    pub examined: u64,
    pub skipped: u64,
    pub processed: u64,
    pub errors: u64,
    pub bytes_per_scan: u64,
    pub pass_start: u64,
    pub scrub_pause: u64,
    pub scrub_spent_paused: u64,
    pub issued_bytes_per_scan: u64,
    pub issued: u64,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, EnumDisplay, EnumIter)]
#[serde(rename_all = "UPPERCASE")]
pub enum ScanFunction {
    None,
    Scrub,
    Resilver,
    ErrorScrub,
    #[serde(other)]
    Unknown,
}

#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, PartialEq, EnumIter)]
#[serde(rename_all = "UPPERCASE")]
pub enum ScanState {
    None,
    Scanning,
    Finished,
    Canceled,
    ErrorScrubbing,
    #[serde(other)]
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RemovalInfo {
    pub state: ScanState,
    pub removing_vdev: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_time: DateTime<Utc>,
    pub to_copy: u64,
    pub copied: u64,
    pub mapping_memory: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CheckpointInfo {
    pub state: CheckpointState,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    pub space: u64,
}

#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CheckpointState {
    None,
    Exists,
    Discarding,
    #[serde(other)]
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RaidzExpandInfo {
    pub state: ScanState,
    pub expanding_vdev: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_time: DateTime<Utc>,
    pub to_reflow: u64,
    pub reflowed: u64,
    pub waiting_for_resilver: bool,
}

#[derive(Debug, Deserialize)]
pub struct VDev {
    pub name: String,
    pub vdev_type: String,
    #[allow(dead_code)]
    pub guid: u64,
    pub path: Option<String>,
    pub class: VDevClass,
    pub state: PoolState,
    pub alloc_space: Option<u64>,
    pub total_space: Option<u64>,
    pub def_space: Option<u64>,
    pub rep_dev_size: Option<u64>,
    pub ex_dev_size: Option<u64>,
    pub self_healed: Option<u64>,
    pub phys_space: Option<u64>,
    pub read_errors: Option<u64>,
    pub write_errors: Option<u64>,
    pub checksum_errors: Option<u64>,
    pub scan_processed: Option<u64>,
    pub checkpoint_space: Option<u64>,
    pub resilver_deferred: Option<u64>,
    pub slow_ios: Option<u64>,
    pub trim_state: Option<TrimState>,
    pub trimmed: Option<u64>,
    pub to_trim: Option<u64>,
    pub trim_time: Option<u64>,
    pub trim_errors: Option<u64>,
    pub trim_notsup: Option<u64>,
    #[serde(default, rename = "vdevs")]
    pub children: HashMap<String, VDev>,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VDevType {
    Root,
    Mirror,
    Replacing,
    Raidz,
    Draid,
    Dspare,
    Disk,
    File,
    Missing,
    Hole,
    Spare,
    Log,
    L2Cache,
    Indirect,
    #[serde(other)]
    Unknown,
}

#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VDevClass {
    Normal,
    Special,
    Hole,
    Spare,
    Log,
    L2Cache,
    #[serde(other)]
    Unknown,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum VDevState {
    Closed,
    Offline,
    Removed,
    CantOpen,
    Faulted,
    Degraded,
    Online,
    #[serde(other)]
    Unknown,
}

#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, PartialEq, EnumIter)]
#[serde(rename_all = "UPPERCASE")]
pub enum TrimState {
    None,
    Active,
    Canceled,
    Suspended,
    Complete,
    #[serde(other)]
    Unknown,
}

fn parse_pool_status(content: &str) -> anyhow::Result<ZpoolStatus> {
    let status: ZpoolStatus =
        serde_json::from_str(content).context("Failed to parse zpool status JSON")?;
    Ok(status)
}

pub async fn get_pool_status() -> anyhow::Result<ZpoolStatus> {
    let output = Command::new("zpool")
        .args(["status", "-j", "-p", "-t", "--json-int"])
        .output()
        .await
        .context("Failed to execute zpool status")?;

    if !output.status.success() {
        anyhow::bail!(
            "zpool status failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let content = String::from_utf8(output.stdout).context("Failed to parse zpool list JSON")?;
    parse_pool_status(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_zpool_status_fixture() {
        let json_data = include_str!("fixtures/zpool_status.json");
        let result: Result<ZpoolStatus, _> = parse_pool_status(json_data);

        match result {
            Ok(status) => {
                println!("Successfully deserialized {} pools", status.pools.len());
                for pool in &status.pools {
                    println!("Pool: {:?}", pool.1);
                }
            }
            Err(e) => {
                panic!("Failed to deserialize zpool status fixture: {}", e);
            }
        }
    }
}
