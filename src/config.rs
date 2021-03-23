use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::path::PathBuf;

type SubvolMap = HashMap<String, Subvolume>;

#[derive(Debug, Deserialize)]
pub struct Subvolume {
    pub mountpoint: String,
    pub path: PathBuf,
    #[serde(default = "usize_max")]
    pub hourly_limit: usize,
    #[serde(default = "usize_max")]
    pub daily_limit: usize,
    #[serde(default = "usize_max")]
    pub weekly_limit: usize,
    #[serde(default = "usize_max")]
    pub monthly_limit: usize,
    #[serde(default = "usize_max")]
    pub yearly_limit: usize,
}

impl Subvolume {
    pub fn escaped_mountpoint(&self) -> String {
        crate::util::escape_slash(&self.mountpoint)
    }
}

fn deserialize_subv<'a, D: Deserializer<'a>>(d: D) -> Result<SubvolMap, D::Error> {
    let mut subvs = <Vec<Subvolume>>::deserialize(d)?;
    Ok(subvs.drain(..).map(|x| (x.mountpoint.clone(), x)).collect())
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub device: PathBuf,
    pub mount_option: Vec<String>,
    pub snapshot_root: PathBuf,
    #[serde(default = "suffix_default")]
    pub suffix: String,
    #[serde(deserialize_with = "deserialize_subv")]
    pub subvolumes: SubvolMap,
}

fn suffix_default() -> String {
    String::from("-auto")
}

fn usize_max() -> usize {
    usize::MAX
}

pub fn get_config(file: &str) -> anyhow::Result<Config> {
    let buf = std::fs::read_to_string(file)?;
    Ok(toml::from_str(&buf)?)
}
