use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::config::Config;
use crate::util::{escape_slash, run_cmd};

const DATE_FORMAT: &str = "%FT%H-%M-%S";

pub fn list(config: &Config, filesystem: &str) -> Result<()> {
    let subdir = config.snapshot_root.join(escape_slash(filesystem));
    fs::read_dir(&subdir)
        .with_context(|| format!("Cannot read sub directory {}", subdir.display()))?
        .try_for_each::<_, io::Result<()>>(|snap| {
            println!("{}", snap?.path().display());
            Ok(())
        })
        .with_context(|| format!("Cannot read an entry inside {}", subdir.display()))
}

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
    // ensures that suffix begins with a dash
    let suffix_dash = if suffix.starts_with('-') { "" } else { "-" };
    let name = format!("{}{}{}", now, suffix_dash, suffix);
    let subv = config
        .subvolumes
        .get(filesystem)
        .with_context(|| format!("Filesystem {} not found in config", filesystem))?;
    log::info!("Snapshoting {}", filesystem);
    make_snapshot(config, &subv.path, &escape_slash(&subv.filesystem), &name)?;
    Ok(())
}

pub fn create_all(config: &Config, suffix: &str) -> Result<()> {
    config
        .subvolumes
        .values()
        .filter_map(|sv| sv.create.then(|| &sv.filesystem))
        .try_for_each(|fs| create(config, suffix, fs))
}

fn sorted_suffixed_snap_date(subdir: &Path, suffix: &str) -> Result<Vec<(PathBuf, NaiveDateTime)>> {
    let mut snap_date: Vec<_> = fs::read_dir(&subdir)
        .with_context(|| format!("Cannot read sub directory {}", subdir.display()))?
        .filter_map(|d| match d {
            Ok(d) => {
                let path = d.path();
                let name = path.file_name()?.to_str()?;
                let datestr = name
                    .ends_with(suffix)
                    .then(|| name.trim_end_matches(suffix))?;
                let date = NaiveDateTime::parse_from_str(datestr, DATE_FORMAT).ok()?;
                Some(Ok((path, date)))
            }
            Err(e) => Some(Err(e)),
        })
        .collect::<io::Result<_>>()
        .with_context(|| format!("Cannot read an entry inside {}", subdir.display()))?;
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
                log::info!("Deleting subvolume {}", path.display());
                run_cmd(&[&"btrfs", &"subvolume", &"delete", path])?;
            } else {
                eprintln!("Will delete subvolume {}", path.display());
            }
        }
    }
    Ok(())
}

pub fn autoclean(config: &Config, filesystem: &str, dryrun: bool) -> Result<()> {
    let subv = config
        .subvolumes
        .get(filesystem)
        .with_context(|| format!("Filesystem {} not found in config", filesystem))?;
    let subdir = config.snapshot_root.join(escape_slash(filesystem));
    let snap_date = sorted_suffixed_snap_date(&subdir, "-auto")?;

    log::info!("Auto cleaning {}", filesystem);
    let mut num_hourly = 0;
    let mut num_daily = 0;
    let mut num_weekly = 0;
    let mut num_monthly = 0;
    let mut num_yearly = 0;
    let mut prev: Option<(&PathBuf, &NaiveDateTime)> = None;
    for (snap, date) in snap_date.iter() {
        use chrono::{Datelike, Timelike};
        macro_rules! is_last_in {
            ($field:ident) => {
                prev.map(|(_, prevd)| prevd.$field() != date.$field())
                    .unwrap_or(true)
            };
        }

        if num_hourly < subv.hourly_limit && is_last_in!(hour) {
            num_hourly += 1;
        } else if num_daily < subv.daily_limit && is_last_in!(day) {
            num_daily += 1;
        } else if num_weekly < subv.weekly_limit && is_last_in!(iso_week) {
            num_weekly += 1;
        } else if num_monthly < subv.monthly_limit && is_last_in!(month) {
            num_monthly += 1;
        } else if num_yearly < subv.yearly_limit && is_last_in!(year) {
            num_yearly += 1;
        } else {
            // Don't keep this snapshot
            if !dryrun {
                log::info!("Deleting subvolume {}", snap.display());
                run_cmd(&[&"btrfs", &"subvolume", &"delete", snap])?;
            } else {
                eprintln!("Will delete subvolume {}", snap.display());
            }
        }
        prev = Some((snap, date));
    }
    Ok(())
}

pub fn autoclean_all(config: &Config, dryrun: bool) -> Result<()> {
    config
        .subvolumes
        .values()
        .filter_map(|sv| sv.autoclean.then(|| &sv.filesystem))
        .try_for_each(|fs| autoclean(config, fs, dryrun))
}
