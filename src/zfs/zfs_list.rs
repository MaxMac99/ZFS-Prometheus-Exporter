use anyhow::Context;
use enum_display::EnumDisplay;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use tokio::process::Command;

// JSON structures for zpool list output
#[derive(Debug, Deserialize)]
pub struct ZfsList {
    #[serde(default)]
    pub pools: HashMap<String, Dataset>,
}

#[derive(Debug, Deserialize)]
pub struct Dataset {
    pub name: String,
    #[serde(rename = "type")]
    pub dataset_type: DatasetType,
    pub pool: String,
    pub properties: Properties,
}

#[derive(Copy, Clone, Debug, Deserialize, EnumDisplay, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DatasetType {
    Filesystem,
    Volume,
    Snapshot,
    Pool,
    Bookmark,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub used: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub available: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_u64")]
    pub referenced: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_bool")]
    pub mounted: Option<bool>,
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
fn deserialize_optional_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let prop = PropertyValue::<serde_json::Value>::deserialize(deserializer)?;
    match prop.value {
        serde_json::Value::String(s) => match s.as_str() {
            "yes" => Ok(Some(true)),
            "no" => Ok(Some(false)),
            "-" => Ok(None),
            _ => Err(DeError::custom("expected 'yes', 'no', or '-'")),
        },
        _ => Ok(None),
    }
}

fn parse_dataset_list(content: &str) -> anyhow::Result<ZfsList> {
    let dataset_list: ZfsList =
        serde_json::from_str(content).context("Failed to parse zfs list JSON")?;

    Ok(dataset_list)
}

pub async fn get_dataset_list() -> anyhow::Result<ZfsList> {
    let output = Command::new("zfs")
        .args([
            "list",
            "-Hpj",
            "--json-int",
            "-o",
            "name,used,available,referenced,compressratio,mounted",
        ])
        .output()
        .await
        .context("Failed to execute zfs list")?;

    if !output.status.success() {
        anyhow::bail!(
            "zfs list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let content = String::from_utf8(output.stdout).context("Failed to parse zpool list JSON")?;
    parse_dataset_list(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_zfs_list_fixture() {
        let json_data = include_str!("fixtures/zfs_list.json");
        let result: Result<ZfsList, _> = parse_dataset_list(json_data);

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
