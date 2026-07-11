use stores::algorithms::fuzzy::*;

#[test]
fn test_levenshtein_dist(){
    let min_accepted_percentage = 0.5;
    let s1 = "Street Fighter";
    
    // Fail
    let mut s2 = "Magic Scroll Tactics";
    let mut dist = levenshtein_distance(s1, s2);
    println!("Distance is {}", dist);
    let percentage = 1.0 - (dist/s1.len() as f32);
    println!("Percentage: {}", percentage);
    assert_ne!(true, percentage >= min_accepted_percentage, "{} should be less than {}", percentage, min_accepted_percentage);

    // Pass
    s2 = "Street Fighter IV";
    dist = levenshtein_distance(s1, s2);
    println!("Distance is {}", dist);
    let percentage = 1.0 - (dist/s1.len() as f32);
    println!("Percentage: {}", percentage);
    assert_eq!(true, percentage >= min_accepted_percentage, "{} should be less than {}", percentage, min_accepted_percentage);
}

#[test]
fn test_damerau_levenshtein(){
    let min_accepted_percentage = 0.5;
    let s1 = "Street Fighter";

    // Fail
    let mut s2 = "Magic Scroll Tactics";
    let mut dist = damerau_levenshtein(s1, s2);
    println!("Distance is {}", dist);
    let percentage = 1.0 - (dist/s1.len() as f32);
    println!("Percentage: {}", percentage);
    assert_ne!(true, percentage >= min_accepted_percentage, "{} should be less than {}", percentage, min_accepted_percentage);

    // Pass
    s2 = "Street Fighter IV";
    dist = damerau_levenshtein(s1, s2);
    println!("Distance is {}", dist);
    let percentage = 1.0 - (dist/s1.len() as f32);
    println!("Percentage: {}", percentage);
    assert_eq!(true, percentage >= min_accepted_percentage, "{} should be less than {}", percentage, min_accepted_percentage);
}

#[test]
fn test_smith_waterman(){
    let s1 = "Street Fighter";
    let min_accepted_percentage = 0.75;

    // Fail
    let mut s2 = "Rune Fighter";
    let mut score = smith_waterman(s1, s2);
    println!("Score is {}", score);
    let mut percent = score/(2*s1.len()) as f32;
    println!("Percentage: {}", percent);
    assert_eq!(true, percent < min_accepted_percentage,"{} should be less than {}", percent, min_accepted_percentage);

    // Pass
    s2 = "Street Fighter IV";
    score = smith_waterman(s1, s2);
    println!("Distance is {}", score);
    percent = score/(2*s1.len()) as f32;
    println!("Percentage: {}", percent);
    assert_eq!(true, percent > min_accepted_percentage, "{} should be greater than {}", percent, min_accepted_percentage);
}