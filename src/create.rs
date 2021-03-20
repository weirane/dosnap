use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::util::{escape_slash, run_cmd};

fn make_snapshot(config: &Config, subvol: &Path, path_escape: &str, name: &str) -> Result<()> {
    let dst = config.snapshot_root.join(path_escape).join(name);
    if dst.exists() {
        log::warn!("Destination {} exists, ignored", dst.display());
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    run_cmd(&[&"btrfs", &"subvolume", &"snapshot", &"-r", &subvol, &dst])
}

pub fn create(config: &Config, matches: &clap::ArgMatches) -> Result<()> {
    let now = chrono::offset::Local::now().format("%FT%H-%M-%S");
    let name = format!("{}{}", now, config.suffix);
    let to_snap = matches.values_of("filesystem").unwrap();
    for (mnt_point, subv) in to_snap.flat_map(|fs| config.subvolumes.get_key_value(fs)) {
        log::info!("Snapshoting {}", mnt_point);
        let path = escape_slash(mnt_point);
        make_snapshot(config, subv, &path, &name)?;
    }
    Ok(())
}
