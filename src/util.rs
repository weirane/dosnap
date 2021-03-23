use anyhow::{anyhow, Context, Result};
use std::ffi::OsStr;
use std::io;
use std::process::{Command, Stdio};

pub fn escape_slash(path: &str) -> String {
    path.replace("%", "%%").replace("/", "%")
}

/// Extracts the suffix from an ArgMatches. The AUTO flag is honored.
pub fn get_suffix<'a>(matches: &'a clap::ArgMatches) -> Option<&'a str> {
    if matches.is_present("AUTO") {
        Some("-auto")
    } else {
        matches.value_of("SUFFIX")
    }
}

pub fn run_cmd(cmd: &[&dyn AsRef<OsStr>]) -> Result<()> {
    let cmd_vec: Vec<_> = cmd.iter().map(|s| s.as_ref().to_string_lossy()).collect();
    log::debug!("executing {:?}", cmd_vec);

    let res = match cmd {
        [] => Err(io::Error::new(io::ErrorKind::Other, "empty command")),
        [cmd] => Command::new(cmd).stdout(Stdio::inherit()).output(),
        [cmd, args @ ..] => Command::new(cmd)
            .args(args)
            .stdout(Stdio::inherit())
            .output(),
    };
    match res {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                let errmsg = String::from_utf8_lossy(&o.stderr).trim().to_string();
                Err(anyhow!(errmsg)).context(format!("command {:?} failed", cmd_vec))
            }
        }
        Err(e) => Err(e).context(format!("command {:?} failed", cmd_vec)),
    }
}
