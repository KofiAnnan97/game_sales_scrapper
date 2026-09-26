use std::time::Instant;
use stores::algorithms::fuzzy::*;

const MIN_ACCEPTABLE_PERCENT: f32 = 0.6;

#[test]
fn test_levenshtein_dist() {
    let s1 = "Street Fighter";

    // Fail
    let mut s2 = "Magic Scroll Tactics";
    let mut start = Instant::now();
    let mut dist = levenshtein_distance(s1, s2);
    let mut elapsed_time = start.elapsed();
    println!("\nDistance is {}", dist);
    let percentage = 1.0 - (dist / s1.len() as f32);
    println!("Percentage: {}", percentage);
    println!("Elapsed Time {} (in nanosecs)", elapsed_time.as_nanos());
    assert_eq!(
        Some(std::cmp::Ordering::Less),
        percentage.partial_cmp(&MIN_ACCEPTABLE_PERCENT),
        "{} should be less than {}",
        percentage,
        MIN_ACCEPTABLE_PERCENT
    );

    // Pass
    s2 = "Street Fighter IV";
    start = Instant::now();
    dist = levenshtein_distance(s1, s2);
    elapsed_time = start.elapsed();
    println!("\nDistance is {}", dist);
    let percentage = 1.0 - (dist / s1.len() as f32);
    println!("Percentage: {}", percentage);
    println!("Elapsed Time {} (in nanosecs)", elapsed_time.as_nanos());
    assert!(
        percentage >= MIN_ACCEPTABLE_PERCENT,
        "{} should be less than {}",
        percentage,
        MIN_ACCEPTABLE_PERCENT
    );
}

#[test]
fn test_damerau_levenshtein() {
    let s1 = "Street Fighter";

    // Fail
    let mut s2 = "Magic Scroll Tactics";
    let mut start = Instant::now();
    let mut dist = damerau_levenshtein(s1, s2);
    let mut elapsed_time = start.elapsed();
    println!("\nDistance is {}", dist);
    let percentage = 1.0 - (dist / s1.len() as f32);
    println!("Percentage: {}", percentage);
    println!("Elapsed Time {} (in nanosecs)", elapsed_time.as_nanos());
    assert!(
        percentage < MIN_ACCEPTABLE_PERCENT,
        "{} should be less than {}",
        percentage,
        MIN_ACCEPTABLE_PERCENT
    );

    // Pass
    s2 = "Street Fighter IV";
    start = Instant::now();
    dist = damerau_levenshtein(s1, s2);
    elapsed_time = start.elapsed();
    println!("\nDistance is {}", dist);
    let percentage = 1.0 - (dist / s1.len() as f32);
    println!("Percentage: {}", percentage);
    println!("Elapsed Time {} (in nanosecs)", elapsed_time.as_nanos());
    assert!(
        percentage >= MIN_ACCEPTABLE_PERCENT,
        "{} should be less than {}",
        percentage,
        MIN_ACCEPTABLE_PERCENT
    );
}

#[test]
fn test_smith_waterman() {
    let s1 = "Street Fighter";

    // Fail
    let mut s2 = "Rune Fighter";
    let mut start = Instant::now();
    let mut score = smith_waterman(s1, s2);
    let mut elapsed_time = start.elapsed();
    println!("\nDistance is {}", score);
    let mut percent = score / (2 * s1.len()) as f32;
    println!("Percentage: {}", percent);
    println!("Elapsed Time {} (in nanosecs)", elapsed_time.as_nanos());
    assert!(
        percent < MIN_ACCEPTABLE_PERCENT,
        "{} should be less than {}",
        percent,
        MIN_ACCEPTABLE_PERCENT
    );

    // Pass
    s2 = "Street Fighter IV";
    start = Instant::now();
    score = smith_waterman(s1, s2);
    elapsed_time = start.elapsed();
    println!("\nDistance is {}", score);
    percent = score / (2 * s1.len()) as f32;
    println!("Percentage: {}", percent);
    println!("Elapsed Time {} (in nanosecs)", elapsed_time.as_nanos());
    assert!(
        percent > MIN_ACCEPTABLE_PERCENT,
        "{} should be greater than {}",
        percent,
        MIN_ACCEPTABLE_PERCENT
    );
}

fn long_algorithm_inputs() -> (String, String) {
    let query = "Street Fighter ".repeat(640);
    let reference = format!(
        "{}{}{}{}{}{}",
        "Street Fighter ".repeat(68),
        "ZZZZZZZZZZZZZZZ".repeat(110),
        "Street Fighter ".repeat(300),
        "ZZZZZZZZZZZZZZZ".repeat(24),
        "Street Fighter ".repeat(100),
        "ZZZZZZZZZZZZZZZ".repeat(38),
    );

    (query, reference)
}

#[test]
fn test_levenshtein_dist_with_extremely_long_strings() {
    let (query, reference) = long_algorithm_inputs();
    let start = Instant::now();
    let distance = levenshtein_distance(&query, &reference);
    let elapsed_time = start.elapsed();
    let percent = 1.0 - (distance / query.len() as f32);

    println!("\nDistance is {}", distance);
    println!("Percentage: {}", percent);
    println!("Elapsed Time {} (in millis)", elapsed_time.as_millis());

    assert!(
        percent > MIN_ACCEPTABLE_PERCENT,
        "{} should be greater than {}",
        percent,
        MIN_ACCEPTABLE_PERCENT
    );
}

#[test]
fn test_damerau_levenshtein_with_extremely_long_strings() {
    let (query, reference) = long_algorithm_inputs();
    let start = Instant::now();
    let distance = damerau_levenshtein(&query, &reference);
    let elapsed_time = start.elapsed();
    let percent = 1.0 - (distance / query.len() as f32);

    println!("\nDistance is {}", distance);
    println!("Percentage: {}", percent);
    println!("Elapsed Time {} (in millis)", elapsed_time.as_millis());

    assert!(
        percent > MIN_ACCEPTABLE_PERCENT,
        "{} should be greater than {}",
        percent,
        MIN_ACCEPTABLE_PERCENT
    );
}

#[test]
fn test_smith_waterman_with_extremely_long_strings() {
    let (query, reference) = long_algorithm_inputs();
    let start = Instant::now();
    let score = smith_waterman(&query, &reference);
    let elapsed_time = start.elapsed();
    let percent = score / (2 * query.len()) as f32;

    println!("\nDistance is {}", score);
    println!("Percentage: {}", percent);
    println!("Elapsed Time {} (in millis)", elapsed_time.as_millis());

    assert!(
        percent > MIN_ACCEPTABLE_PERCENT,
        "{} should be greater than {}",
        percent,
        MIN_ACCEPTABLE_PERCENT
    );
}
