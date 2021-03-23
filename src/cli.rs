use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    let dryrun = Arg::with_name("DRYRUN")
        .short("d")
        .long("dry-run")
        .help("Don't actually perform the deletion");
    App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg_from_usage("-c, --config=[FILE] 'Sets a custom config file'")
        .subcommand(
            SubCommand::with_name("list")
                .about("List snapshots of a filesystem")
                .arg_from_usage("<filesystem> 'Filesystem to list'"),
        )
        .subcommand(
            SubCommand::with_name("create")
                .about("Creates a snapshot")
                .arg(
                    Arg::with_name("SUFFIX")
                        .short("s")
                        .long("suffix")
                        .default_value("-manual")
                        .help("Suffix of the snapshot name"),
                )
                .arg(
                    Arg::with_name("AUTO")
                        .long("auto")
                        .help("Set suffix to '-auto', short for --suffix=-auto"),
                )
                .arg_from_usage("<filesystem> 'Filesystem to snapshot'"),
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("Cleans the snapshots")
                .arg(
                    Arg::with_name("SUFFIX")
                        .short("s")
                        .long("suffix")
                        .required(true)
                        .takes_value(true)
                        .help("Suffix of the snapshot name"),
                )
                .arg(
                    Arg::with_name("NKEEP")
                        .short("n")
                        .long("nkeep")
                        .takes_value(true)
                        .help("Keep n snapshots"),
                )
                .arg(&dryrun)
                .arg_from_usage("<filesystem> 'Filesystem to clean'"),
        )
        .subcommand(
            SubCommand::with_name("autoclean")
                .about("Auto clean according to the limits")
                .arg(&dryrun)
                .arg_from_usage("<filesystem> 'Filesystem to clean'"),
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("Generate completion")
                .arg(
                    Arg::with_name("SHELL")
                        .short("s")
                        .long("shell")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["bash", "fish", "zsh", "powershell", "elvish"])
                        .help("Generate completion for SHELL"),
                ),
        )
}

pub fn gen_completion(shell: &str) {
    use clap::Shell::*;
    let shell = match shell {
        "bash" => Bash,
        "fish" => Fish,
        "zsh" => Zsh,
        "powershell" => PowerShell,
        "elvish" => Elvish,
        _ => panic!("Invalid shell: {}", shell),
    };
    build_cli().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut std::io::stdout());
}
