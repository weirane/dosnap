use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::path::PathBuf;

type SubvolMap = HashMap<String, PathBuf>;

#[derive(Debug, Deserialize)]
struct Subvolume {
    mountpoint: String,
    path: PathBuf,
}

fn deserialize_subv<'a, D: Deserializer<'a>>(d: D) -> Result<SubvolMap, D::Error> {
    use std::iter::FromIterator;
    let mut subvs = <Vec<Subvolume>>::deserialize(d)?;
    let ret = HashMap::from_iter(subvs.drain(..).map(|x| (x.mountpoint, x.path)));
    Ok(ret)
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

pub fn get_config(file: &str) -> anyhow::Result<Config> {
    let buf = std::fs::read_to_string(file)?;
    Ok(toml::from_str(&buf)?)
}