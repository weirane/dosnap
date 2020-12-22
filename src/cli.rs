use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg_from_usage("-c, --config=[FILE] 'Sets a custom config file'")
        .subcommand(
            SubCommand::with_name("create")
                .about("create a snapshot")
                .arg_from_usage("<filesystem>... 'filesystems to snapshot'"),
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("clean snapshots")
                .arg_from_usage("-n, --nkeep=[NUM] 'Keep n snapshots'"),
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("generate completion")
                .arg(
                    Arg::with_name("SHELL")
                        .short("s")
                        .long("shell")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["bash", "fish", "zsh", "powershell", "elvish"])
                        .help("generate completion for SHELL"),
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