use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Once,
    },
};
use coreutilsp::utils::clap_ext::CommandExt;
use clap::{CommandFactory, Parser};
use rayon::iter::{Either, IntoParallelRefIterator, ParallelBridge, ParallelIterator};

#[derive(Parser, Clone, Debug)]
#[command(version, disable_help_flag = true, disable_version_flag = true)]
struct Cli {
    #[arg(short = 'R', visible_short_alias = 'r', long = "recursive")]
    recursive: bool,

    #[arg(short = 'f', long = "force")]
    force: bool,

    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    #[arg(short = 'p')]
    preserve: bool,

    #[arg(short = 'P')]
    no_dereference: bool,

    #[arg(short = 'H')]
    dereference_args: bool,

    #[arg(short = 'L')]
    dereference: bool,

    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    #[arg()]
    files: Vec<String>,
}

struct State {
    cli: Cli,
    has_any_error: AtomicBool,
}

type AppState = Arc<State>;

impl State {
    fn should_follow_symlink(&self, is_arg: bool) -> bool {
        if self.cli.recursive {
            if self.cli.dereference {
                true // -L
            } else if self.cli.dereference_args {
                is_arg // -H
            } else {
                false // -P or default
            }
        } else {
            !self.cli.no_dereference
        }
    }

    fn should_overwrite(&self, dest: &Path) -> bool {
        if self.cli.interactive {
            eprint!("cp-par: overwrite '{}'? ", dest.display());
            io::stderr().flush().expect("failed to flush stderr");
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).expect("failed to read from console");
            matches!(buf.chars().next(), Some('y') | Some('Y'))
        } else {
            true
        }
    }
}

// --- Platform abstraction ---

#[cfg(unix)]
fn is_special_file(meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::FileTypeExt;
    let ft = meta.file_type();
    ft.is_fifo() || ft.is_block_device() || ft.is_char_device() || ft.is_socket()
}

#[cfg(not(unix))]
fn is_special_file(_meta: &fs::Metadata) -> bool {
    false
}

#[cfg(unix)]
fn create_symlink(original: &Path, link: &Path, _is_dir: bool) -> io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn create_symlink(original: &Path, link: &Path, is_dir: bool) -> io::Result<()> {
    if is_dir {
        std::os::windows::fs::symlink_dir(original, link)
    } else {
        std::os::windows::fs::symlink_file(original, link)
    }
}

#[cfg(not(any(unix, windows)))]
fn create_symlink(_original: &Path, _link: &Path, _is_dir: bool) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "symlink creation not supported on this platform",
    ))
}

/// Try to preserve ownership. Returns true if successful (or not applicable).
#[cfg(unix)]
fn preserve_ownership(path: &Path, src_meta: &fs::Metadata, follow: bool) -> bool {
    use std::os::unix::fs::MetadataExt;
    let uid = Some(src_meta.uid());
    let gid = Some(src_meta.gid());
    if follow {
        std::os::unix::fs::chown(path, uid, gid).is_ok()
    } else {
        std::os::unix::fs::lchown(path, uid, gid).is_ok()
    }
}

#[cfg(not(unix))]
fn preserve_ownership(_path: &Path, _src_meta: &fs::Metadata, _follow: bool) -> bool {
    true
}

/// Set permissions, clearing SUID/SGID if chown failed on Unix.
fn preserve_permissions(path: &Path, src_meta: &fs::Metadata, chown_ok: bool) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let mode = if chown_ok {
            src_meta.mode() & 0o7777
        } else {
            src_meta.mode() & 0o1777 // clear SUID/SGID
        };
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
    #[cfg(not(unix))]
    {
        let _ = chown_ok;
        fs::set_permissions(path, src_meta.permissions())
    }
}

fn preserve_timestamps(path: &Path, src_meta: &fs::Metadata) -> io::Result<()> {
    let mut times = fs::FileTimes::new();
    if let Ok(t) = src_meta.accessed() {
        times = times.set_accessed(t);
    }
    if let Ok(t) = src_meta.modified() {
        times = times.set_modified(t);
    }
    // Try write mode first (needed on Windows), fall back to read-only (works for Unix dirs)
    let f = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .or_else(|_| fs::File::open(path))?;
    f.set_times(times)
}

/// Ensure owner can write into a newly created directory during copy.
fn set_dir_owner_writable(path: &Path, src_meta: &fs::Metadata) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        fs::set_permissions(path, fs::Permissions::from_mode(src_meta.mode() | 0o700))
    }
    #[cfg(not(unix))]
    {
        let _ = (path, src_meta);
        Ok(()) // directories are writable by default on Windows
    }
}

// --- Core logic ---

fn is_same_file(a: &Path, b: &Path) -> bool {
    match (fs::metadata(a), fs::metadata(b)) {
        #[cfg(unix)]
        (Ok(ma), Ok(mb)) => {
            use std::os::unix::fs::MetadataExt;
            ma.dev() == mb.dev() && ma.ino() == mb.ino()
        }
        #[cfg(not(unix))]
        (Ok(_), Ok(_)) => {
            // Best effort: compare canonical paths
            match (fs::canonicalize(a), fs::canonicalize(b)) {
                (Ok(ca), Ok(cb)) => ca == cb,
                _ => false,
            }
        }
        _ => false,
    }
}

fn dest_is_descendant_of_src(src: &Path, dest: &Path) -> bool {
    let src_canon = match fs::canonicalize(src) {
        Ok(p) => p,
        Err(_) => return false,
    };
    match fs::canonicalize(dest) {
        Ok(dest_canon) => dest_canon.starts_with(&src_canon) && dest_canon != src_canon,
        Err(_) => {
            // dest doesn't exist yet; walk up to find an existing ancestor
            let mut check = dest.to_path_buf();
            loop {
                if let Ok(canon) = fs::canonicalize(&check) {
                    return canon.starts_with(&src_canon);
                }
                if !check.pop() {
                    return false;
                }
            }
        }
    }
}

fn preserve_attributes(path: &Path, src_meta: &fs::Metadata, is_symlink: bool) -> io::Result<()> {
    if is_symlink {
        // For symlinks: just try ownership (timestamps/perms not meaningful)
        preserve_ownership(path, src_meta, false);
        return Ok(());
    }

    let chown_ok = preserve_ownership(path, src_meta, true);
    preserve_permissions(path, src_meta, chown_ok)?;
    preserve_timestamps(path, src_meta)?;

    Ok(())
}

fn copy_file(state: &State, src: &Path, dest: &Path, src_meta: &fs::Metadata) -> io::Result<()> {
    if dest.symlink_metadata().is_ok() && !state.should_overwrite(dest) {
        return Ok(());
    }

    match fs::copy(src, dest) {
        Ok(_) => {}
        Err(_) if state.cli.force => {
            let _ = fs::remove_file(dest);
            fs::copy(src, dest)?;
        }
        Err(e) => return Err(e),
    }

    if state.cli.preserve {
        preserve_attributes(dest, src_meta, false)?;
    }

    Ok(())
}

fn copy_symlink(state: &State, src: &Path, dest: &Path, src_meta: &fs::Metadata) -> io::Result<()> {
    if dest.symlink_metadata().is_ok() {
        if !state.should_overwrite(dest) {
            return Ok(());
        }
        fs::remove_file(dest).or_else(|_| fs::remove_dir(dest))?;
    }

    let link_target = fs::read_link(src)?;
    // Determine if target is a directory (needed for Windows symlinks)
    let target_is_dir = fs::metadata(src).map(|m| m.is_dir()).unwrap_or(false);
    create_symlink(&link_target, dest, target_is_dir)?;

    if state.cli.preserve {
        preserve_attributes(dest, src_meta, true)?;
    }

    Ok(())
}

fn copy_dir(state: AppState, src: &Path, dest: &Path, src_meta: &fs::Metadata) -> io::Result<()> {
    match dest.symlink_metadata() {
        Ok(dm) if dm.is_dir() => {}
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "cannot overwrite non-directory '{}' with directory '{}'",
                    dest.display(),
                    src.display()
                ),
            ));
        }
        Err(_) => {
            fs::create_dir(dest)?;
            // Ensure owner can write during copy, final permissions set after
            set_dir_owner_writable(dest, src_meta)?;
        }
    }

    let src_owned = src.to_owned();
    let dest_owned = dest.to_owned();
    let local_error = Once::new();

    fs::read_dir(&src_owned)?.par_bridge().for_each(|item| {
        match item {
            Ok(entry) => {
                let name = entry.file_name();
                let child_src = src_owned.join(&name);
                let child_dest = dest_owned.join(&name);
                copy_entry(state.clone(), &child_src, &child_dest, false);
            }
            Err(err) => {
                local_error.call_once(|| {
                    state.has_any_error.store(true, Ordering::Release);
                    eprintln!("cp-par: cannot read directory '{}': {}", src_owned.display(), err);
                });
            }
        }
    });

    if state.cli.preserve {
        preserve_attributes(dest, src_meta, false)?;
    } else {
        fs::set_permissions(dest, src_meta.permissions())?;
    }

    Ok(())
}

fn copy_entry_internal(state: AppState, src: &Path, dest: &Path, is_arg: bool) -> io::Result<()> {
    let follow = state.should_follow_symlink(is_arg);
    let meta = if follow { fs::metadata(src) } else { fs::symlink_metadata(src) }
        .map_err(|e| io::Error::new(e.kind(), format!("cannot stat '{}': {}", src.display(), e)))?;

    if is_same_file(src, dest) {
        Err(io::Error::new(io::ErrorKind::Other,
            format!("'{}' and '{}' are the same file", src.display(), dest.display())))?;
    }

    if meta.is_dir() {
        if !state.cli.recursive {
            Err(io::Error::new(io::ErrorKind::Other,
                format!("-R not specified; omitting directory '{}'", src.display())))?;
        }
        if is_arg && dest_is_descendant_of_src(src, dest) {
            Err(io::Error::new(io::ErrorKind::Other,
                format!("cannot copy a directory, '{}', into itself, '{}'", src.display(), dest.display())))?;
        }
        copy_dir(state.clone(), src, dest, &meta)
    } else if meta.file_type().is_symlink() {
        copy_symlink(&state, src, dest, &meta)
    } else if state.cli.recursive && is_special_file(&meta) {
        Err(io::Error::new(io::ErrorKind::Unsupported,
            format!("cannot copy special file '{}': not supported", src.display())))
    } else {
        copy_file(&state, src, dest, &meta)
    }
}

fn copy_entry(state: AppState, src: &Path, dest: &Path, is_arg: bool) {
    match copy_entry_internal(state.clone(), src, dest, is_arg) {
        Ok(()) => {
            if state.cli.verbose {
                println!("'{}' -> '{}'", src.display(), dest.display());
            }
        }
        Err(err) => {
            state.has_any_error.store(true, Ordering::Release);
            if err.kind() == io::ErrorKind::Other {
                eprintln!("cp-par: {}", err);
            } else {
                eprintln!("cp-par: cannot copy '{}' to '{}': {}", src.display(), dest.display(), err);
            }
        }
    }
}

fn main() -> ExitCode {
    let cli: Cli = match Cli::command().help_version_long_only().parse() {
        Either::Left(cli) => cli,
        Either::Right(exit_code) => return exit_code,
    };

    if cli.files.len() < 2 {
        if cli.files.is_empty() {
            eprintln!("cp-par: missing file operand");
        } else {
            eprintln!("cp-par: missing destination file operand after '{}'", cli.files[0]);
        }
        eprintln!("Try 'cp-par --help' for more information.");
        return ExitCode::FAILURE;
    }

    let sources: Vec<String> = cli.files[..cli.files.len() - 1].to_vec();
    let target = PathBuf::from(&cli.files[cli.files.len() - 1]);
    let target_is_dir = target.is_dir();

    if sources.len() > 1 && !target_is_dir {
        eprintln!("cp-par: target '{}' is not a directory", target.display());
        return ExitCode::FAILURE;
    }

    let state = Arc::new(State {
        cli,
        has_any_error: AtomicBool::new(false),
    });

    sources.par_iter().for_each(|src| {
        let src_path = PathBuf::from(src);
        let dest = if target_is_dir {
            match src_path.file_name() {
                Some(name) => target.join(name),
                None => {
                    state.has_any_error.store(true, Ordering::Release);
                    eprintln!("cp-par: cannot determine filename of '{}'", src);
                    return;
                }
            }
        } else {
            target.clone()
        };
        copy_entry(state.clone(), &src_path, &dest, true);
    });

    if state.has_any_error.load(Ordering::Acquire) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
