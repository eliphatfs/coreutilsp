use std::iter::zip;
use coreutilsp::utils::size_unit::{parse_size, format_size};


#[test]
fn test_parse_size() {
    let test_cases = vec![
        "1024",     // plain int
        "1k",       // 1024
        "1K",       // 1024
        "1kB",      // 1000
        "1kiB",     // 1024
        "1G",       // 1073741824
        "1g",       // error (invalid suffix)
        "1Z",       // error (too large for i64)
        "-1m",      // -1048576
        "10MB",     // 10000000
        "10MiB",    // 10485760
        "1.5K",    // error (floating point)
    ];
    let results = vec![
        Some(1024),
        Some(1024),
        Some(1024),
        Some(1000),
        Some(1024),
        Some(1073741824),
        None,
        None,
        Some(-1048576),
        Some(10000000),
        Some(10485760),
        None,
    ];
    for (case, result) in zip(test_cases, results) {
        match (parse_size(case), result) {
            (Ok(x), Some(y)) => { assert_eq!(x, y, "{} parsed size mismatch", case); },
            (Ok(x), None) => { assert!(false, "{} got {}, expected returning error", case, x) },
            (Err(err), Some(y)) => { assert!(false, "{} got {}, expected returning {}", case, err, y) },
            (Err(_), None) => { },
        }
    }
}

#[test]
fn test_format_size() {
    assert_eq!(format_size(0), "0");
    assert_eq!(format_size(2), "2");
    assert_eq!(format_size(1023), "1023");
    assert_eq!(format_size(1024), "1.0K");
    assert_eq!(format_size(1025), "1.1K");
    assert_eq!(format_size(2048), "2.0K");
    assert_eq!(format_size(2049), "2.1K");
    assert_eq!(format_size(10137), "9.9K");
    assert_eq!(format_size(10138), "10K");
    assert_eq!(format_size(20480), "20K");
    assert_eq!(format_size(21480), "21K");
    assert_eq!(format_size(25480), "25K");
    assert_eq!(format_size(1047552), "1023K");
    assert_eq!(format_size(1047553), "1.0M");
    assert_eq!(format_size(1048576), "1.0M");
    assert_eq!(format_size(1048577), "1.1M");
    assert_eq!(format_size(1073741824), "1.0G");
    assert_eq!(format_size(1099511627776), "1.0T");
}
