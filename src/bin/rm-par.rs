use std::{fs, io::{self, Write}, path::PathBuf, process::ExitCode, sync::{atomic::{AtomicBool, Ordering}, Arc, Once}};
use coreutilsp::utils::clap_ext::CommandExt;
use coreutilsp::utils::work_entry::{WorkEntry, WorkEntryPathExt};
use clap::{CommandFactory, Parser};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator, Either};

#[derive(Parser, Clone, Debug)] // requires `derive` feature
#[command(version, disable_help_flag = true, disable_version_flag = true)]
struct Cli {
    #[arg(short = 'f', long = "force")]
    force: bool,
    
    #[arg(short = 'I')]
    interactive_once: bool,

    #[arg(short = 'r', visible_short_alias = 'R', long = "recursive")]
    recursive: bool,

    #[arg(short = 'd', long = "dir")]
    rmdir: bool,

    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    #[arg()]
    files: Vec<String>,
}

struct State {
    cli: Cli,
    has_any_error: AtomicBool
}

type AppState = Arc<State>;

fn handle_entry_internal(state: AppState, entry: &impl WorkEntry, depth: u32) -> io::Result<bool> {
    match entry.is_dir()? {
        true => Ok({
            let local_error_logic= Once::new();
            let path = entry.path();
            if state.cli.recursive {
                if path.is_root() {
                    Err(io::Error::new(io::ErrorKind::Other, format!("it is dangerous to operate recursively on '{}'", path.display())))?;
                }
                else if path.is_curdir_or_parent() {
                    Err(io::Error::new(io::ErrorKind::Other, format!("refusing to remove '.' or '..' directory: skipping '{}'", path.display())))?;
                }
                fs::read_dir(&path)?.par_bridge().for_each(|item| {
                    match item {
                        Ok(entry) => {
                            handle_entry(state.clone(), &entry, depth + 1);
                        },
                        Err(err) => {
                            if err.kind() != io::ErrorKind::NotFound || !state.cli.force {
                                local_error_logic.call_once(|| {
                                    state.has_any_error.store(true, Ordering::Release);
                                    eprintln!("rm-par: cannot read directory '{}': {}", path.display(), err);
                                });
                            }
                        }
                    }
                });
                fs::remove_dir(entry.path())?;
            }
            else if state.cli.rmdir {
                if path.is_curdir_or_parent() {
                    Err(io::Error::new(io::ErrorKind::Other, format!("refusing to remove '.' or '..' directory: skipping '{}'", path.display())))?;
                }
                fs::remove_dir(entry.path())?;
            }
            else {
                Err(io::Error::new(io::ErrorKind::IsADirectory, "Is a directory"))?
            }
            true
        }),
        false => Ok({
            // generic file, including regular file, device, symlink, etc.
            fs::remove_file(entry.path())?;
            false
        })
    }
}

fn handle_entry(state: AppState, entry: &impl WorkEntry, depth: u32) -> bool {
    match handle_entry_internal(state.clone(), entry, depth) {
        Ok(is_dir) => {
            if state.cli.verbose {
                if is_dir {
                    println!("removed directory '{}'", entry.path().display());
                }
                else {
                    println!("removed '{}'", entry.path().display());
                }
            }
            is_dir
        },
        Err(err) => {
            if err.kind() != io::ErrorKind::NotFound || !state.cli.force {
                state.has_any_error.store(true, Ordering::Release);
                if err.kind() == io::ErrorKind::Other {
                    eprintln!("rm-par: {}", err);
                }
                else {
                    eprintln!("rm-par: cannot remove '{}': {}", entry.path().display(), err);
                }
            }
            false
        }
    }
}

fn main() -> ExitCode {
    let cli: Cli = match Cli::command().help_version_long_only().parse() {
        Either::Left(cli) => cli,
        Either::Right(exit_code) => return exit_code
    };

    if cli.files.len() == 0 {
        eprintln!("rm-par: missing operand");
        eprintln!("Try 'rm-par --help' for more information.");
        return ExitCode::FAILURE;
    }

    if cli.interactive_once && (cli.recursive || cli.files.len() > 3) {
        let arg = if cli.files.len() == 1 { "argument" } else { "arguments" };
        if cli.recursive {
            eprint!("rm-par: remove {} {} recursively? ", cli.files.len(), arg);
        }
        else {
            eprint!("rm-par: remove {} {}? ", cli.files.len(), arg);
        }
        io::stderr().flush().expect("fail to print prompt message");
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).expect("fail to read from console");
        match buf.chars().nth(0) {
            Some('y') | Some('Y') => { },
            _ => { return ExitCode::SUCCESS }
        }
    }

    let state = Arc::new(State { cli: cli, has_any_error: AtomicBool::new(false) });

    state.cli.files.par_iter().for_each(|x| {
        let path = PathBuf::from(x);
        handle_entry(state.clone(), &path, 0);
    });

    if state.has_any_error.load(Ordering::Acquire) {
        ExitCode::FAILURE
    }
    else {
        ExitCode::SUCCESS
    }
}
