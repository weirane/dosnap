mod action;
mod cli;
mod config;
mod util;

use anyhow::Context;
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::config::{get_config, Config};
use crate::util::{get_suffix, run_cmd};

fn setup(config: &Config) -> anyhow::Result<(PathBuf, TempDir)> {
    use nix::sched::{unshare, CloneFlags};

    unshare(CloneFlags::CLONE_NEWNS).context("unable to unshare")?;
    run_cmd(&[&"mount", &"--make-rprivate", &"/"])?;

    let btrfs_mnt = tempfile::Builder::new()
        .prefix("btrfs-snapshot-")
        .tempdir()?;
    let mount_options = config.mount_option.join(",");
    run_cmd(&[
        &"mount",
        &"-o",
        &mount_options,
        &config.device,
        &btrfs_mnt.path(),
    ])?;

    let cwd = env::current_dir()?;
    env::set_current_dir(btrfs_mnt.path())?;
    Ok((cwd, btrfs_mnt))
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = cli::build_cli().get_matches();

    if let Some(matches) = opts.subcommand_matches("completion") {
        cli::gen_completion(matches.value_of("SHELL").unwrap());
        return Ok(());
    }

    let config = get_config(opts.value_of("config").unwrap_or("/etc/dosnap.toml"))
        .context("Get config failed")?;
    if !nix::unistd::geteuid().is_root() {
        eprintln!("Please run as root");
        std::process::exit(1);
    }
    let (cwd, tempdir) = setup(&config).context("Setup failed")?;

    match opts.subcommand() {
        ("list", Some(matches)) => {
            let filesystem = matches.value_of("filesystem").unwrap();
            action::list(&config, filesystem).context("List failed")?;
        }
        ("create", Some(matches)) => {
            let suffix = get_suffix(matches).unwrap();
            let filesystem = matches.value_of("filesystem").unwrap();
            action::create(&config, suffix, filesystem).context("Create failed")?;
        }
        ("clean", Some(matches)) => {
            let nkeep: usize = matches
                .value_of("NKEEP")
                .unwrap()
                .parse()
                .context("Cannot parse nkeep")?;
            let filesystem = matches.value_of("filesystem").unwrap();
            let suffix = get_suffix(matches).unwrap();
            let dryrun = matches.is_present("DRYRUN");
            action::clean(&config, suffix, filesystem, nkeep, dryrun).context("Clean failed")?;
        }
        ("autoclean", Some(matches)) => {
            let dryrun = matches.is_present("DRYRUN");
            if matches.is_present("ALL") {
                action::autoclean_all(&config, dryrun).context("Auto clean all failed")?;
            } else {
                let filesystem = matches.value_of("filesystem").unwrap();
                action::autoclean(&config, filesystem, dryrun).context("Auto clean failed")?;
            }
        }
        _ => {}
    }

    env::set_current_dir(&cwd)?;
    nix::mount::umount(tempdir.path())?;

    Ok(())
}
