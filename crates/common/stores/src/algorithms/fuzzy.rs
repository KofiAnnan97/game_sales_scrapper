use std::cmp;

fn print_matrix(matrix: &Vec<Vec<usize>>) {
     println!("Levenshtein Distance Matrix:");
     for line in matrix{
        println!("{:?}", line);
     }
}

pub fn levenstein_dist(input: &str, expected: &str) -> f32 {
    let input_chars: Vec<char> = input.chars().collect(); 
    let expected_chars: Vec<char> = expected.chars().collect(); 

    let input_len: usize =  input_chars.len();
    let expected_len: usize = expected_chars.len();

    let mut cmp_matrix = vec![vec![0usize; expected_len+1]; input_len+1];
    for i in 0..=input_len {
        cmp_matrix[i][0] = i;
    }
    for j in 0..=expected_len {
        cmp_matrix[0][j] = j;
    }

    for i in  1..=input_len {
        for j in 1..=expected_len {
            let cost = if input_chars[i-1] == expected_chars[j-1] {
                0
            } else {
                1
            };
            cmp_matrix[i][j] = cmp::min(
                cmp::min(
                    cmp_matrix[i-1][j] + 1, 
                    cmp_matrix[i][j-1] + 1
                ), 
                cmp_matrix[i-1][j-1] + cost
            );
        }
    }

    #[cfg(test)]
    print_matrix(&cmp_matrix);
    
    cmp_matrix[input_len][expected_len] as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenstein_dist_fails(){
        let s1 = "Street Fghter";
        let s2 = "Magic Scroll Tactics";
        let min_accepted_percentage = 0.5;
        let dist = levenstein_dist(s1, s2);
        println!("Distance is {}", dist);
        let percentage = 1.0 - (dist/s1.len() as f32);
        println!("Percentage: {}", percentage);
        assert_ne!(true, percentage >= min_accepted_percentage, "{} should be less than {}", percentage, min_accepted_percentage);
    }
}

