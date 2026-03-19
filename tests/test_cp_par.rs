use std::{fs, os, path::{Path, PathBuf}, process::Command};

fn cp_par() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cp-par"))
}

/// Create a fresh temp directory under the system temp dir, unique per test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cp_par_test_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ---- Single file copy ----

#[test]
fn test_copy_single_file() {
    let dir = test_dir("single_file");
    fs::write(dir.join("src.txt"), "hello").unwrap();

    let status = cp_par().arg(dir.join("src.txt")).arg(dir.join("dst.txt")).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(dir.join("dst.txt")).unwrap(), "hello");
}

#[test]
fn test_copy_overwrites_existing() {
    let dir = test_dir("overwrite");
    fs::write(dir.join("src.txt"), "new").unwrap();
    fs::write(dir.join("dst.txt"), "old").unwrap();

    let status = cp_par().arg(dir.join("src.txt")).arg(dir.join("dst.txt")).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(dir.join("dst.txt")).unwrap(), "new");
}

// ---- Multiple sources into directory ----

#[test]
fn test_copy_multiple_into_dir() {
    let dir = test_dir("multi_into_dir");
    fs::write(dir.join("a.txt"), "aaa").unwrap();
    fs::write(dir.join("b.txt"), "bbb").unwrap();
    fs::create_dir(dir.join("dest")).unwrap();

    let status = cp_par()
        .arg(dir.join("a.txt"))
        .arg(dir.join("b.txt"))
        .arg(dir.join("dest"))
        .status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(dir.join("dest/a.txt")).unwrap(), "aaa");
    assert_eq!(fs::read_to_string(dir.join("dest/b.txt")).unwrap(), "bbb");
}

#[test]
fn test_copy_single_into_existing_dir() {
    let dir = test_dir("single_into_dir");
    fs::write(dir.join("f.txt"), "data").unwrap();
    fs::create_dir(dir.join("dest")).unwrap();

    let status = cp_par().arg(dir.join("f.txt")).arg(dir.join("dest")).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(dir.join("dest/f.txt")).unwrap(), "data");
}

// ---- Recursive copy ----

#[test]
fn test_recursive_copy() {
    let dir = test_dir("recursive");
    fs::create_dir_all(dir.join("src/sub")).unwrap();
    fs::write(dir.join("src/a.txt"), "aaa").unwrap();
    fs::write(dir.join("src/sub/b.txt"), "bbb").unwrap();

    let status = cp_par().arg("-R").arg(dir.join("src")).arg(dir.join("dst")).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(dir.join("dst/a.txt")).unwrap(), "aaa");
    assert_eq!(fs::read_to_string(dir.join("dst/sub/b.txt")).unwrap(), "bbb");
}

#[test]
fn test_recursive_into_existing_dir() {
    let dir = test_dir("recursive_into");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/f.txt"), "data").unwrap();
    fs::create_dir(dir.join("dest")).unwrap();

    let status = cp_par().arg("-R").arg(dir.join("src")).arg(dir.join("dest")).status().unwrap();
    assert!(status.success());
    // Should create dest/src/f.txt (nested inside existing dir)
    assert_eq!(fs::read_to_string(dir.join("dest/src/f.txt")).unwrap(), "data");
}

#[test]
fn test_recursive_deep_nesting() {
    let dir = test_dir("deep_nesting");
    fs::create_dir_all(dir.join("src/a/b/c")).unwrap();
    fs::write(dir.join("src/a/b/c/leaf.txt"), "deep").unwrap();

    let status = cp_par().arg("-R").arg(dir.join("src")).arg(dir.join("dst")).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(dir.join("dst/a/b/c/leaf.txt")).unwrap(), "deep");
}

#[test]
fn test_recursive_merges_existing_dest_dir() {
    let dir = test_dir("merge");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/new.txt"), "new").unwrap();
    fs::create_dir_all(dir.join("dest/src")).unwrap();
    fs::write(dir.join("dest/src/old.txt"), "old").unwrap();

    let status = cp_par().arg("-R").arg(dir.join("src")).arg(dir.join("dest")).status().unwrap();
    assert!(status.success());
    // Both old and new files should exist
    assert_eq!(fs::read_to_string(dir.join("dest/src/old.txt")).unwrap(), "old");
    assert_eq!(fs::read_to_string(dir.join("dest/src/new.txt")).unwrap(), "new");
}

// ---- Directory without -R ----

#[test]
fn test_dir_without_recursive_fails() {
    let dir = test_dir("dir_no_r");
    fs::create_dir(dir.join("src")).unwrap();

    let output = cp_par().arg(dir.join("src")).arg(dir.join("dst")).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("omitting directory"), "stderr: {}", stderr);
}

// ---- Same file detection ----

#[test]
fn test_same_file_fails() {
    let dir = test_dir("same_file");
    fs::write(dir.join("f.txt"), "data").unwrap();

    let output = cp_par().arg(dir.join("f.txt")).arg(dir.join("f.txt")).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("same file"), "stderr: {}", stderr);
}

// ---- Copy into self detection ----

#[test]
fn test_copy_dir_into_self_fails() {
    let dir = test_dir("into_self");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/f.txt"), "data").unwrap();

    let output = cp_par()
        .arg("-R")
        .arg(dir.join("src"))
        .arg(dir.join("src/inside"))
        .output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("into itself"), "stderr: {}", stderr);
}

// ---- Symlink handling ----

#[cfg(unix)]
#[test]
fn test_symlink_default_follows() {
    let dir = test_dir("symlink_follow");
    fs::write(dir.join("target.txt"), "real").unwrap();
    os::unix::fs::symlink(dir.join("target.txt"), dir.join("link")).unwrap();

    let status = cp_par().arg(dir.join("link")).arg(dir.join("copy.txt")).status().unwrap();
    assert!(status.success());
    // Should copy the contents, not the link
    assert_eq!(fs::read_to_string(dir.join("copy.txt")).unwrap(), "real");
    assert!(!dir.join("copy.txt").symlink_metadata().unwrap().file_type().is_symlink());
}

#[cfg(unix)]
#[test]
fn test_symlink_no_dereference() {
    let dir = test_dir("symlink_noderef");
    fs::write(dir.join("target.txt"), "real").unwrap();
    os::unix::fs::symlink("target.txt", dir.join("link")).unwrap();

    let status = cp_par().arg("-P").arg(dir.join("link")).arg(dir.join("copy")).status().unwrap();
    assert!(status.success());
    // Should copy the symlink itself
    assert!(dir.join("copy").symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(dir.join("copy")).unwrap(), Path::new("target.txt"));
}

#[cfg(unix)]
#[test]
fn test_recursive_preserves_symlinks_by_default() {
    let dir = test_dir("recursive_symlinks");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/real.txt"), "data").unwrap();
    os::unix::fs::symlink("real.txt", dir.join("src/link")).unwrap();

    let status = cp_par().arg("-R").arg(dir.join("src")).arg(dir.join("dst")).status().unwrap();
    assert!(status.success());
    // With -R, default is -P (no dereference) — symlinks copied as symlinks
    assert!(dir.join("dst/link").symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(dir.join("dst/link")).unwrap(), Path::new("real.txt"));
}

#[cfg(unix)]
#[test]
fn test_recursive_dereference_all() {
    let dir = test_dir("recursive_deref_L");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/real.txt"), "data").unwrap();
    os::unix::fs::symlink("real.txt", dir.join("src/link")).unwrap();

    let status = cp_par().arg("-R").arg("-L").arg(dir.join("src")).arg(dir.join("dst")).status().unwrap();
    assert!(status.success());
    // -L: follow all symlinks — link should become a regular file
    assert!(!dir.join("dst/link").symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(fs::read_to_string(dir.join("dst/link")).unwrap(), "data");
}

// ---- Preserve attributes (-p) ----

#[test]
fn test_preserve_timestamps() {
    use std::time::{Duration, SystemTime};

    let dir = test_dir("preserve_ts");
    fs::write(dir.join("src.txt"), "data").unwrap();
    // Set a known old timestamp
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    let times = fs::FileTimes::new().set_modified(old_time).set_accessed(old_time);
    fs::File::options().write(true).open(dir.join("src.txt")).unwrap().set_times(times).unwrap();

    let status = cp_par().arg("-p").arg(dir.join("src.txt")).arg(dir.join("dst.txt")).status().unwrap();
    assert!(status.success());

    let src_mtime = fs::metadata(dir.join("src.txt")).unwrap().modified().unwrap();
    let dst_mtime = fs::metadata(dir.join("dst.txt")).unwrap().modified().unwrap();
    assert_eq!(src_mtime, dst_mtime);
}

#[cfg(unix)]
#[test]
fn test_preserve_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = test_dir("preserve_perms");
    fs::write(dir.join("src.txt"), "data").unwrap();
    fs::set_permissions(dir.join("src.txt"), fs::Permissions::from_mode(0o754)).unwrap();

    let status = cp_par().arg("-p").arg(dir.join("src.txt")).arg(dir.join("dst.txt")).status().unwrap();
    assert!(status.success());

    let dst_mode = fs::metadata(dir.join("dst.txt")).unwrap().permissions().mode() & 0o777;
    assert_eq!(dst_mode, 0o754);
}

// ---- Force (-f) ----

#[cfg(unix)]
#[test]
fn test_force_overwrites_readonly() {
    use std::os::unix::fs::PermissionsExt;

    let dir = test_dir("force_readonly");
    fs::write(dir.join("src.txt"), "new").unwrap();
    fs::write(dir.join("dst.txt"), "old").unwrap();
    fs::set_permissions(dir.join("dst.txt"), fs::Permissions::from_mode(0o444)).unwrap();

    // Without -f: may fail (depends on user; root always succeeds)
    // With -f: should always succeed by unlinking first
    let status = cp_par().arg("-f").arg(dir.join("src.txt")).arg(dir.join("dst.txt")).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(dir.join("dst.txt")).unwrap(), "new");
}

// ---- Verbose (-v) ----

#[test]
fn test_verbose_output() {
    let dir = test_dir("verbose");
    fs::write(dir.join("src.txt"), "data").unwrap();

    let output = cp_par().arg("-v").arg(dir.join("src.txt")).arg(dir.join("dst.txt")).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("->"), "stdout: {}", stdout);
}

// ---- Error cases ----

#[test]
fn test_missing_operand() {
    let output = cp_par().output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing file operand"), "stderr: {}", stderr);
}

#[test]
fn test_missing_destination() {
    let dir = test_dir("missing_dest");
    fs::write(dir.join("f.txt"), "data").unwrap();

    let output = cp_par().arg(dir.join("f.txt")).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing destination"), "stderr: {}", stderr);
}

#[test]
fn test_nonexistent_source() {
    let dir = test_dir("nonexistent_src");

    let output = cp_par().arg(dir.join("nope")).arg(dir.join("dst")).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot stat") || stderr.contains("No such file"), "stderr: {}", stderr);
}

#[test]
fn test_multi_source_target_not_dir() {
    let dir = test_dir("multi_not_dir");
    fs::write(dir.join("a.txt"), "a").unwrap();
    fs::write(dir.join("b.txt"), "b").unwrap();
    fs::write(dir.join("c.txt"), "c").unwrap();

    let output = cp_par()
        .arg(dir.join("a.txt"))
        .arg(dir.join("b.txt"))
        .arg(dir.join("c.txt"))
        .output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a directory"), "stderr: {}", stderr);
}

// ---- Continue on error (partial failure) ----

#[test]
fn test_continues_on_error() {
    let dir = test_dir("continue_err");
    fs::write(dir.join("good.txt"), "ok").unwrap();
    fs::create_dir(dir.join("dest")).unwrap();

    let output = cp_par()
        .arg(dir.join("nonexistent"))
        .arg(dir.join("good.txt"))
        .arg(dir.join("dest"))
        .output().unwrap();
    // Should fail overall (nonexistent source)
    assert!(!output.status.success());
    // But the good file should still be copied
    assert_eq!(fs::read_to_string(dir.join("dest/good.txt")).unwrap(), "ok");
}

// ---- Empty directory ----

#[test]
fn test_recursive_empty_dir() {
    let dir = test_dir("empty_dir");
    fs::create_dir(dir.join("src")).unwrap();

    let status = cp_par().arg("-R").arg(dir.join("src")).arg(dir.join("dst")).status().unwrap();
    assert!(status.success());
    assert!(dir.join("dst").is_dir());
}

// ---- Large file (basic) ----

#[test]
fn test_copy_larger_file() {
    let dir = test_dir("larger_file");
    let data: Vec<u8> = (0..1_000_000u32).flat_map(|i| i.to_le_bytes()).collect();
    fs::write(dir.join("big.bin"), &data).unwrap();

    let status = cp_par().arg(dir.join("big.bin")).arg(dir.join("copy.bin")).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read(dir.join("copy.bin")).unwrap(), data);
}
