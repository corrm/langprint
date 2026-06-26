//! Identifier case conversion, shared by every backend's idiomatic-renaming pass.
//!
//! Each function splits an identifier into words on `_`, `-`, spaces, and case boundaries
//! (including `XMLHttp` → `XML`, `Http`), then re-joins them in the requested convention.

/// Split an identifier into lowercase words on separators and case boundaries.
fn words(name: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();

    let chars: Vec<char> = name.chars().collect();
    for (index, &ch) in chars.iter().enumerate() {
        if ch == '_' || ch == '-' || ch == ' ' {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
            continue;
        }

        let prev = if index > 0 {
            Some(chars[index - 1])
        } else {
            None
        };
        let next = chars.get(index + 1).copied();

        // Boundary before an uppercase letter that starts a new word:
        // lower→Upper (fooBar) or Upper→Upper followed by lower (XMLHttp → XML|Http).
        let starts_word = ch.is_uppercase()
            && match (prev, next) {
                (Some(p), _) if p.is_lowercase() || p.is_ascii_digit() => true,
                (Some(p), Some(n)) if p.is_uppercase() && n.is_lowercase() => true,
                _ => false,
            };

        if starts_word && !current.is_empty() {
            result.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }

    if !current.is_empty() {
        result.push(current);
    }

    result.iter().map(|word| word.to_lowercase()).collect()
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Convert an identifier to `snake_case`.
pub fn to_snake_case(name: &str) -> String {
    words(name).join("_")
}

/// Convert an identifier to `PascalCase`.
pub fn to_pascal_case(name: &str) -> String {
    words(name).iter().map(|word| capitalize(word)).collect()
}

/// Convert an identifier to `camelCase`.
pub fn to_camel_case(name: &str) -> String {
    words(name)
        .iter()
        .enumerate()
        .map(|(index, word)| {
            if index == 0 {
                word.clone()
            } else {
                capitalize(word)
            }
        })
        .collect()
}
