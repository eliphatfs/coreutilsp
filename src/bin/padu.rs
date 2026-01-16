use std::{fs, io, path::PathBuf, process::ExitCode, sync::{atomic::{AtomicBool, Ordering}, Arc, Once}};
use coreutilsp::utils::clap_ext::CommandExt;
use coreutilsp::utils::size_unit::{parse_size, format_size};
use coreutilsp::utils::work_entry::WorkEntry;
use clap::{CommandFactory, FromArgMatches, Parser};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};


#[derive(Parser, Clone, Debug)] // requires `derive` feature
#[command(version, disable_help_flag = true, disable_version_flag = true)]
struct Cli {
    #[arg(short = 'a', long = "all")]
    all: bool,

    #[arg(short = 'h', long = "human-readable")]
    human_readable: bool,

    #[arg(short = 's', long = "summarize")]
    summarize: bool,

    #[arg(short = 'd', long = "max-depth")]
    max_depth: Option<u32>,

    #[arg(short = 'S', long = "separate-dirs")]
    separate_dirs: bool,

    #[arg(short = 'c', long = "total")]
    total: bool,

    #[arg(short = 't', long = "threshold", value_parser = parse_size, default_value_t = 0)]
    threshold: i64,

    #[arg()]
    files: Vec<String>,
}

struct State {
    cli: Cli,
    has_any_error: AtomicBool
}

type AppState = Arc<State>;

fn handle_entry_internal(state: AppState, entry: &impl WorkEntry, depth: u32) -> io::Result<(bool, u64)> {
    match entry.is_dir()? {
        true => Ok({
            let local_error_logic= Once::new();
            let path = entry.path();
            (true, fs::read_dir(&path)?.par_bridge().map(|item| {
                match item {
                    Ok(entry) => {
                        let (sub_isdir, sub_sz) = handle_entry(state.clone(), &entry, depth + 1);  // we have new entry
                        if sub_isdir && state.cli.separate_dirs { 0 } else { sub_sz }
                    },
                    Err(err) => {
                        local_error_logic.call_once(|| {
                            state.has_any_error.store(true, Ordering::Release);
                            eprintln!("padu: cannot read directory '{}': {}", path.display(), err);
                        });
                        0
                    }
                }
            }).sum())
        }),
        false => Ok({
            // generic file, including regular file, device, symlink, etc.
            (false, filesize::file_real_size(entry.path())?)
        })
    }
}

fn report_entry(state: &State, entry: impl std::fmt::Display, sz: u64) {
    if state.cli.human_readable {
        println!("{}\t{}", format_size(sz), entry);
    } else {
        println!("{}\t{}", sz.div_ceil(1024), entry);
    }
}

fn handle_entry(state: AppState, entry: &impl WorkEntry, depth: u32) -> (bool, u64) {
    match handle_entry_internal(state.clone(), entry, depth) {
        Ok((is_dir, sz)) => {
            let th: i64 = state.cli.threshold;
            let abth = th.unsigned_abs();
            let max_depth = if state.cli.summarize { 0 } else { state.cli.max_depth.unwrap_or(depth) };
            let report = (depth <= max_depth) && (depth == 0 || is_dir || state.cli.all) && (if th < 0 { sz <= abth } else { sz >= abth });
            if report {
                report_entry(&state, entry.path().display(), sz);
            }
            (is_dir, sz)
        },
        Err(err) => {
            state.has_any_error.store(true, Ordering::Release);
            eprintln!("padu: cannot access '{}': {}", entry.path().display(), err);
            (false, 0)  // no size is found
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::command().help_version_long_only();
    let mut fmtcmd = cli.clone();
    let mut args = match cli.try_get_matches() {
        Ok(args) => args,
        Err(err) => {
            err.print().expect("failed to print cli parsing error message");
            return ExitCode::FAILURE;
        }
    };
    let mut cli = match Cli::from_arg_matches_mut(&mut args) {
        Ok(args) => args,
        Err(err) => {
            err.format(&mut fmtcmd).print().expect("failed to print cli parsing error message");
            return ExitCode::FAILURE;
        }
    };

    if cli.files.len() == 0 {
        cli.files.push(".".to_owned());
    }

    let state = Arc::new(State { cli: cli, has_any_error: AtomicBool::new(false) });

    let total = state.cli.files.par_iter().map(|x| {
        let path = PathBuf::from(x);
        let (_, sz) = handle_entry(state.clone(), &path, 0);
        sz
    }).sum();

    if state.cli.total {
        report_entry(&state, "total", total);
    }

    if state.has_any_error.load(Ordering::Acquire) {
        ExitCode::FAILURE
    }
    else {
        ExitCode::SUCCESS
    }
}
