use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::config::Config;
use crate::util::{escape_slash, run_cmd};

const DATE_FORMAT: &str = "%FT%H-%M-%S";

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

pub fn create(config: &Config, suffix: &str, filesystem: &str) -> Result<()> {
    let now = chrono::offset::Local::now().format(DATE_FORMAT);
    let name = format!("{}{}", now, suffix);
    let subv = config
        .subvolumes
        .get(filesystem)
        .with_context(|| format!("Filesystem {} not found in config", filesystem))?;
    log::info!("Snapshoting {}", filesystem);
    make_snapshot(config, &subv.path, &subv.escaped_mountpoint(), &name)?;
    Ok(())
}

fn sorted_suffixed_snap_date(subdir: &Path, suffix: &str) -> Result<Vec<(PathBuf, NaiveDateTime)>> {
    let mut snap_date: Vec<_> = fs::read_dir(&subdir)
        .context("Cannot read btrfs temp mountpoint")?
        .filter_map(|d| match d {
            Ok(d) => {
                let path = d.path();
                let name = path.file_name()?.to_str()?;
                let datestr = name
                    .ends_with(suffix)
                    .then(|| name.trim_end_matches(suffix))?;
                let date = NaiveDateTime::parse_from_str(datestr, DATE_FORMAT).ok()?;
                Some(Ok((path.clone(), date)))
            }
            Err(e) => Some(Err(e)),
        })
        .collect::<io::Result<_>>()
        .context("Cannot read subdirectory of btrfs temp mountpoint")?;
    snap_date.sort_by(|(_, d1), (_, d2)| d1.cmp(d2).reverse());
    Ok(snap_date)
}

pub fn clean(
    config: &Config,
    suffix: &str,
    filesystem: &str,
    nkeep: usize,
    dryrun: bool,
) -> Result<()> {
    let subdir = config.snapshot_root.join(escape_slash(filesystem));
    let snap_date = sorted_suffixed_snap_date(&subdir, suffix)?;
    if snap_date.len() > nkeep {
        for path in snap_date.iter().skip(nkeep).map(|s| &s.0) {
            if !dryrun {
                run_cmd(&[&"btrfs", &"subvolume", &"delete", path])?;
            } else {
                eprintln!("Deleting subvolume {}", path.display());
            }
        }
    }
    Ok(())
}
