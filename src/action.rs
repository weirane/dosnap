use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use std::path::Path;
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

pub fn create(config: &Config, matches: &clap::ArgMatches) -> Result<()> {
    let now = chrono::offset::Local::now().format(DATE_FORMAT);
    let suffix = matches.value_of("SUFFIX").unwrap();
    let name = format!("{}{}", now, suffix);
    let to_snap = matches.values_of("filesystem").unwrap();
    for (mnt_point, subv) in to_snap.flat_map(|fs| config.subvolumes.get_key_value(fs)) {
        log::info!("Snapshoting {}", mnt_point);
        let path = escape_slash(mnt_point);
        make_snapshot(config, subv, &path, &name)?;
    }
    Ok(())
}

pub fn clean(config: &Config, matches: &clap::ArgMatches) -> Result<()> {
    let to_clean = matches.value_of("filesystem").unwrap();
    let nkeep: usize = matches
        .value_of("NKEEP")
        .unwrap()
        .parse()
        .context("Cannot parse nkeep")?;
    let subdir = config.snapshot_root.join(escape_slash(to_clean));
    let suffix = matches.value_of("SUFFIX").unwrap();
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
    if snap_date.len() > nkeep {
        snap_date.sort_by(|(_, d1), (_, d2)| d1.cmp(d2).reverse());
        let dryrun = matches.is_present("DRYRUN");
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
