use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::i64 as parse_i64,
    combinator::{all_consuming, map, opt},
    error::Error,
    Parser,
};

/// Parses a string into bytes based on specific casing and unit rules.
pub fn parse_size(input: &str) -> Result<i64, String> {
    // 1. Parse the leading integer
    let (remaining, value) = parse_i64::<&str, Error<&str>>(input)
        .map_err(|_| format!("invalid argument '{}'", input))?;

    // If the string ends exactly after the number
    if remaining.is_empty() {
        return Ok(value);
    }

    // 2. Define the suffix parser with explicit type hints to resolve ambiguity
    let mut suffix_parser = all_consuming::<&str, Error<&str>, _>(alt((
        // k/K and m/M (Case-insensitive prefix)
        map((alt((tag("k"), tag("K"))), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
        map((alt((tag("m"), tag("M"))), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
        // G, T, P, E, Z, Y (Strictly Uppercase prefix)
        map((tag("G"), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
        map((tag("T"), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
        map((tag("P"), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
        map((tag("E"), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
        map((tag("Z"), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
        map((tag("Y"), opt(alt((tag("iB"), tag("B"))))), |(_, s)| s),
    )));

    // 3. Execute the parser
    let (_, suffix_opt) = suffix_parser
        .parse(remaining)
        .map_err(|_| format!("invalid suffix in argument '{}'", input))?;

    // 4. Determine Multiplier Base
    // iB or no suffix -> 1024 (binary), B -> 1000 (decimal)
    let base: i128 = match suffix_opt {
        Some("B") => 1000,
        _ => 1024,
    };

    // 5. Determine Power
    let power: u32 = match remaining.chars().next() {
        Some('k' | 'K') => 1,
        Some('m' | 'M') => 2,
        Some('G') => 3,
        Some('T') => 4,
        Some('P') => 5,
        Some('E') => 6,
        Some('Z') => 7, // Added Zetta
        Some('Y') => 8, // Yotta
        _ => 0,
    };

    // 6. Calculate and check for i64 bounds
    let multiplier = base.pow(power);
    let final_val = (value as i128).checked_mul(multiplier);

    match final_val {
        Some(v) if v <= i64::MAX as i128 && v >= i64::MIN as i128 => Ok(v as i64),
        _ => Err(format!("argument '{}' too large", input)),
    }
}
