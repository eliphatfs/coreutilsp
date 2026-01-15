use clap::{Arg, ArgAction, Command};


pub trait CommandExt {
    fn help_version_long_only(self) -> Self;
}

impl CommandExt for Command {
    fn help_version_long_only(self) -> Self {
        self.arg(
            Arg::new("help")
                .long("help") // Only --help
                .action(ArgAction::Help)
                .help("Print help information")
                .global(true) 
        )
        // 3. Add back --version as a long-only argument
        .arg(
            Arg::new("version")
                .long("version") // Only --version
                .action(ArgAction::Version)
                .help("Print version information")
        )
    }
}
