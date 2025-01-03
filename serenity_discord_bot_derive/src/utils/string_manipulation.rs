pub fn pascal_to_snake_case(s: &str) -> String {
    let mut res = String::with_capacity(s.len());

    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        res.push_str(&c.to_lowercase().to_string());
        if let Some(next) = chars.peek() {
            if next.is_uppercase() {
                res.push('_');
            }
        }
    }

    res
}
