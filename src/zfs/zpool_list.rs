use crate::zfs::zpool_status::PoolState;
use anyhow::{Context, Result};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, trace};

// JSON structures for zpool list output
#[derive(Debug, Deserialize)]
pub struct ZpoolList {
    #[serde(default)]
    pub pools: HashMap<String, Pool>,
}

#[derive(Debug, Deserialize)]
pub struct Pool {
    pub name: String,
    pub state: PoolState,
    pub properties: Properties,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub size: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub allocated: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub free: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub checkpoint: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub expandsize: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub fragmentation: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub capacity: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_f64")]
    pub dedupratio: Option<f64>,
    #[serde(deserialize_with = "deserialize_optional_pool_state")]
    pub health: Option<PoolState>,
}

// Helper struct for deserializing property values
#[derive(Deserialize)]
struct PropertyValue<T> {
    value: T,
}

// Custom deserializer for u64 properties that handles "-" as None
fn deserialize_optional_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let prop = PropertyValue::<serde_json::Value>::deserialize(deserializer)?;
    match prop.value {
        serde_json::Value::Number(n) => Ok(Some(
            n.as_u64().ok_or_else(|| DeError::custom("expected u64"))?,
        )),
        serde_json::Value::String(s) if s == "-" => Ok(None),
        _ => Ok(None),
    }
}

// Custom deserializer for f64 properties that handles "-" as None and string numbers
fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let prop = PropertyValue::<serde_json::Value>::deserialize(deserializer)?;
    match prop.value {
        serde_json::Value::Number(n) => Ok(Some(
            n.as_f64().ok_or_else(|| DeError::custom("expected f64"))?,
        )),
        serde_json::Value::String(s) if s == "-" => Ok(None),
        serde_json::Value::String(s) => s
            .parse::<f64>()
            .ok()
            .map(Some)
            .ok_or_else(|| DeError::custom(format!("expected f64, got string: {}", s))),
        _ => Ok(None),
    }
}

// Custom deserializer for PoolState properties
fn deserialize_optional_pool_state<'de, D>(deserializer: D) -> Result<Option<PoolState>, D::Error>
where
    D: Deserializer<'de>,
{
    let prop = PropertyValue::<serde_json::Value>::deserialize(deserializer)?;
    match prop.value {
        serde_json::Value::String(s) if s == "-" => Ok(None),
        serde_json::Value::String(s) => serde_json::from_value(serde_json::Value::String(s))
            .map(Some)
            .map_err(DeError::custom),
        _ => Ok(None),
    }
}

fn parse_pool_list(content: &str) -> Result<ZpoolList> {
    let pool_list: ZpoolList =
        serde_json::from_str(content).context("Failed to parse zpool list JSON")?;
    Ok(pool_list)
}

pub async fn get_pool_list() -> Result<ZpoolList> {
    debug!("Running zpool list -Hpj --json-int to get pool list");
    let output = Command::new("zpool")
        .args(["list", "-Hpj", "--json-int"])
        .output()
        .await
        .context("Failed to execute zpool list")?;

    if !output.status.success() {
        anyhow::bail!(
            "zpool list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    debug!("zpool list command executed successfully");
    trace!("zpool list output: {:?}", &output);

    let content = String::from_utf8(output.stdout).context("Failed to parse zpool list JSON")?;
    parse_pool_list(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_zpool_list_fixture() {
        let json_data = include_str!("fixtures/zpool_list.json");
        let result: Result<ZpoolList, _> = parse_pool_list(json_data);

        match result {
            Ok(status) => {
                println!("Successfully deserialized {} pools", status.pools.len());
                for pool in &status.pools {
                    println!("Pool: {:?}", pool);
                }
            }
            Err(e) => {
                panic!("Failed to deserialize zpool status fixture: {}", e);
            }
        }
    }
}
