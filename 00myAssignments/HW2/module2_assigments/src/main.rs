fn concat_strings(s1: &String, s2: &String) -> String {
    let mut result = s1.clone();
    result.push_str(s2);
    result
}

fn sum_with_step(sum: &mut i32, low: i32, high: i32, step: i32) {
    *sum = 0;
    
    if step <= 0 {
        return;
    }
    
    let mut current = low;
    while current <= high {
        *sum += current;
        current += step;
    }
}

fn most_frequent_word(text: &str) -> (String, usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    
    if words.is_empty() {
        return (String::new(), 0);
    }
    
    let mut max_word = String::new();
    let mut max_count = 0;
    
    for &word in &words {
        let mut count = 0;
        
        for &other_word in &words {
            if word.to_lowercase() == other_word.to_lowercase() {
                count += 1;
            }
        }
        
        if count > max_count {
            max_count = count;
            max_word = word.to_string();
        }
    }
    
    (max_word, max_count)
}

fn main() {
    let s1 = String::from("Hello, ");
    let s2 = String::from("World!");
    let result = concat_strings(&s1, &s2);
    println!("{}", result);

    let mut result = 0;
    
    sum_with_step(&mut result, 1, 10, 1);
    println!("Sum from 1 to 10 with step 1: {}", result);
    
    sum_with_step(&mut result, 2, 20, 2);
    println!("Sum from 2 to 20 with step 2: {}", result);
    
    sum_with_step(&mut result, 5, 15, 3);
    println!("Sum from 5 to 15 with step 3: {}", result);

    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text);
    println!("Most frequent word: \"{}\" ({} times)", word, count);
}
