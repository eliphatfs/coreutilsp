use std::path::{Component, PathBuf};
use coreutilsp::utils::work_entry::WorkEntry;

#[test]
fn test_dots() {
    assert!(PathBuf::from("test/..").is_curdir_or_parent());
    assert!(PathBuf::from("..").is_curdir_or_parent());
    assert!(PathBuf::from(".").is_curdir_or_parent());
    assert!(!PathBuf::from("test").is_curdir_or_parent());
    assert!(!PathBuf::from("1").is_curdir_or_parent());
    assert!(!PathBuf::from("1.").is_curdir_or_parent());
    assert!(!PathBuf::from("1..").is_curdir_or_parent());
    assert!(!PathBuf::from("1..txt").is_curdir_or_parent());
    assert!(PathBuf::from("test/..").components().last() == Some(Component::ParentDir));
    assert!(PathBuf::from("..").components().last() == Some(Component::ParentDir));
    assert!(PathBuf::from(".").components().last() == Some(Component::CurDir));
    assert!(PathBuf::from("").components().last() == None);
    assert!(PathBuf::from("test/test.").components().last() != Some(Component::CurDir));
    assert!(PathBuf::from("test/test..").components().last() != Some(Component::ParentDir));
}

#[test]
fn test_is_root() {
    assert_eq!(PathBuf::from("value").is_root(), false);
    assert_eq!(PathBuf::from("../../../../../../../../../../../../../../../../../../..").is_root(), true);
    assert_eq!(PathBuf::from("../../../../../../../../../../../../../../../../../../../").is_root(), true);
    assert_eq!(PathBuf::from("/").is_root(), true);
    assert_eq!(PathBuf::from("/value").is_root(), false);
    #[cfg(windows)]
    {
        assert_eq!(PathBuf::from("C:/").is_root(), true);
        assert_eq!(PathBuf::from("C:\\").is_root(), true);
    }
}
