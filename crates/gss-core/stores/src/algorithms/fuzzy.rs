use std::{char, cmp};

#[cfg(test)]
fn print_digit_matrix<T: std::fmt::Debug>(title: &str, query: &str, reference: &str, matrix: &Vec<Vec<T>>){
    let mut reference_display = String::from("     ");
    for c in reference.chars() {
        let reference_val_str = format!(" {}", c);
        reference_display.push_str(&reference_val_str);
    }
    let mut horizontal_line = String::from("  ");
    for _ in 2..reference_display.len() {
        horizontal_line.push('-');
    }

    println!("{}\n{}\n{}", title, reference_display, horizontal_line);
    for i in 0..matrix.len(){
        let matrix_row = &matrix[i];
        let mut matrix_row_str = if i == 0 {
            String::from("  |")
        } else if i > query.len() {
            String::new()
        } else {
            String::from(query.chars().nth(i-1).unwrap()) + " |"
        };
        for j in 0..matrix_row.len() {
            let val_str = format!(" {:?}", matrix_row[j]);
            matrix_row_str.push_str(&val_str);
        }
        println!("{}", matrix_row_str);
    }
}

pub fn levenshtein_distance(query: &str, reference: &str) -> f32 {
    let query_chars: Vec<char> = query.chars().collect(); 
    let reference_chars: Vec<char> = reference.chars().collect(); 

    let query_len: usize =  query_chars.len();
    let reference_len: usize = reference_chars.len();

    let mut cmp_matrix = vec![vec![0usize; reference_len+1]; query_len+1];
    for i in 0..=query_len {
        cmp_matrix[i][0] = i;
    }
    for j in 0..=reference_len {
        cmp_matrix[0][j] = j;
    }

    for i in  1..=query_len {
        for j in 1..=reference_len {
            let cost = if query_chars[i-1] == reference_chars[j-1] {
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
    print_digit_matrix("Levenshtein Distance Matrix:", query, reference, &cmp_matrix);
    
    cmp_matrix[query_len][reference_len] as f32
}

pub fn damerau_levenshtein(query: &str, reference: &str) -> f32{
    let query_chars: Vec<char> = query.chars().collect(); 
    let reference_chars: Vec<char> = reference.chars().collect(); 

    let query_len: usize =  query_chars.len();
    let reference_len: usize = reference_chars.len();

    let mut cmp_matrix = vec![vec![0usize; reference_len+1]; query_len+1];
    for i in 0..=query_len {
        cmp_matrix[i][0] = i;
    }
    for j in 0..=reference_len {
        cmp_matrix[0][j] = j;
    }

    for i in  1..=query_len {
        for j in 1..=reference_len {
            let cost = if query_chars[i-1] == reference_chars[j-1] {
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
            if i > 1 && j > 1 && (query_chars[i-1] == reference_chars[j-2]) 
                && (query_chars[i-2] == reference_chars[j-1]) {
                cmp_matrix[i][j] = cmp::min(cmp_matrix[i][j], cmp_matrix[i-2][j-2] + cost);
            } 

        }
    }

    #[cfg(test)]
    print_digit_matrix("Damerau Levenshtein Matrix:", query, reference, &cmp_matrix);

    cmp_matrix[query_len][reference_len] as f32    
}

// Local Sequence Alignment for all possible segments length
pub fn smith_waterman(query: &str, reference: &str) -> f32 {
    let match_score = 2;
    let mismatch_penalty = 1;
    let gap_penalty = 2;

    let mut max_score = 0;
    #[allow(unused_variables)]
    let mut _max_pos = (0, 0);

    let query_chars: Vec<char> = query.chars().collect();
    let reference_chars: Vec<char> = reference.chars().collect();

    let query_len = query_chars.len() + 1;
    let reference_len = reference_chars.len() + 1;

    let mut cmp_matrix = vec![vec![0i32; reference_len]; query_len];

    // Compute matrix
    for i in 1..query_len {
        for j in 1..reference_len {
            let diagonal = if query_chars[i - 1] == reference_chars[j - 1] {
                cmp_matrix[i - 1][j - 1] + match_score
            } else {
                cmp_matrix[i - 1][j - 1] - mismatch_penalty
            };

            let up = cmp_matrix[i - 1][j] - gap_penalty;
            let left = cmp_matrix[i][j - 1] - gap_penalty;

            cmp_matrix[i][j] = 0.max(diagonal).max(up).max(left);

            if cmp_matrix[i][j] > max_score {
                max_score = cmp_matrix[i][j];
                _max_pos = (i, j);
            }
        }
    }
    
    #[cfg(test)]
    println!("{:?}", _max_pos);
    #[cfg(test)]
    let alignments = sw_traceback(&query_chars, &reference_chars, &_max_pos, match_score, gap_penalty, &cmp_matrix);
    #[cfg(test)]
    print_digit_matrix("Smith–Waterman Matrix:", query, reference, &cmp_matrix);
    #[cfg(test)]
    println!("\nQuery alignment: {}\n Reference alignment: {}", alignments.0, alignments.1);

    max_score as f32
}

#[cfg(test)]
fn sw_traceback(query_chars: &Vec<char>, reference_chars: &Vec<char>, max_pos: &(usize, usize), match_score: i32, gap_penalty: i32, cmp_matrix: &Vec<Vec<i32>>) -> (String, String){
    let mut alignment_query = String::new();
    let mut alignment_reference = String::new();
    let mut i = max_pos.0;
    let mut j = max_pos.1;
    let empty_char = '-';
    
    while i > 0 && j > 0 && cmp_matrix[i][j] > 0 {
        if query_chars[i - 1] == reference_chars[j - 1] && cmp_matrix[i][j] == cmp_matrix[i-1][j-1] + match_score {
            alignment_query.insert(0, query_chars[i-1]);
            alignment_reference.insert(0, reference_chars[j-1]);
            i-=1;
            j-=1;
        } else if cmp_matrix[i][j] == cmp_matrix[i-1][j] - gap_penalty {
            alignment_query.insert(0, query_chars[i-1]);
            alignment_reference.insert(0, empty_char);
            i-=1;
        } else if cmp_matrix[i][j] == cmp_matrix[i][j - 1] - gap_penalty{
            alignment_query.insert(0, empty_char);
            alignment_reference.insert(0, reference_chars[j-1]);
            j-=1; 
        } else {
            alignment_query.insert(0, query_chars[i - 1]);
            alignment_reference.insert(0, reference_chars[j - 1]);
            i -= 1;
            j -= 1;
        }
    }

    (alignment_query, alignment_reference)
}