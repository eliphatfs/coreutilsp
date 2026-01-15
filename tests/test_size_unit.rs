use std::iter::zip;
use coreutilsp::utils::size_unit::parse_size;


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
