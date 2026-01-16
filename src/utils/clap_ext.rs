use std::{fmt::Debug, process::ExitCode};
use clap::{Arg, ArgAction, Command, Parser};
use rayon::iter::Either;


pub trait CommandExt {
    fn help_version_long_only(self) -> Self;
    fn parse<T: Parser + Clone + Debug>(self) -> Either<T, ExitCode>;
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

    fn parse<T: Parser + Clone + Debug>(self) -> Either<T, ExitCode> {
        let mut fmtcmd = self.clone();
        let mut args = match self.try_get_matches() {
            Ok(args) => args,
            Err(err) => {
                err.print().expect("failed to print cli parsing error message");
                return Either::Right(if err.use_stderr() { ExitCode::FAILURE } else { ExitCode::SUCCESS });
            }
        };
        let cli = match T::from_arg_matches_mut(&mut args) {
            Ok(args) => args,
            Err(err) => {
                let fmterr = err.format(&mut fmtcmd);
                fmterr.print().expect("failed to print cli parsing error message");
                return Either::Right(if fmterr.use_stderr() { ExitCode::FAILURE } else { ExitCode::SUCCESS });
            }
        };
        Either::Left(cli)
    }
}
