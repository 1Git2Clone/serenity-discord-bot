pub fn upper_lowercase_permutations(data: &str) -> Vec<String> {
    if data.is_empty() {
        return vec![String::new()];
    }

    let first = data.chars().next().unwrap();
    let rest = &data[1..];

    let permutations = upper_lowercase_permutations(rest);

    let mut result: Vec<String> = Vec::new();

    for perm in permutations {
        result.push(format!("{}{}", first.to_ascii_lowercase(), perm));
        result.push(format!("{}{}", first.to_ascii_uppercase(), perm));
    }

    result
}
