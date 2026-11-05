pub(super) fn normalize_severity(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "warning" => "warning".to_string(),
        "critical" => "critical".to_string(),
        _ => "error".to_string(),
    }
}

pub(super) fn more_severe<'a>(left: &'a str, right: &'a str) -> &'a str {
    fn rank(value: &str) -> u8 {
        match value {
            "critical" => 3,
            "error" => 2,
            "warning" => 1,
            _ => 0,
        }
    }
    if rank(right) > rank(left) {
        right
    } else {
        left
    }
}

pub(super) fn truncate(value: &str, max_chars: usize) -> String {
    let mut output: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        output.push('…');
    }
    output
}

pub(super) fn fingerprint(source: &str, message: &str) -> String {
    let normalized = normalize_for_fingerprint(message);
    let fingerprint_input = format!("{source}|{normalized}");
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in fingerprint_input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("ISSUE-{hash:016X}")
}

pub(super) fn normalize_for_fingerprint(message: &str) -> String {
    let lowercase = message.to_lowercase();
    let characters: Vec<char> = lowercase.chars().collect();
    let mut output = String::with_capacity(lowercase.len());
    let mut previous_was_space = false;
    let mut previous_was_digit_marker = false;
    let mut index = 0usize;

    while index < characters.len() {
        let character = characters[index];
        if character.is_ascii_hexdigit() || character == '-' {
            let start = index;
            let mut hyphen_count = 0usize;
            while index < characters.len()
                && (characters[index].is_ascii_hexdigit() || characters[index] == '-')
            {
                if characters[index] == '-' {
                    hyphen_count += 1;
                }
                index += 1;
            }
            if index - start >= 20 && hyphen_count >= 2 {
                output.push_str("<id>");
                previous_was_space = false;
                previous_was_digit_marker = false;
                continue;
            }
            for value in &characters[start..index] {
                append_normalized_character(
                    &mut output,
                    *value,
                    &mut previous_was_space,
                    &mut previous_was_digit_marker,
                );
            }
            continue;
        }

        append_normalized_character(
            &mut output,
            character,
            &mut previous_was_space,
            &mut previous_was_digit_marker,
        );
        index += 1;
    }

    output.trim().to_string()
}

pub(super) fn append_normalized_character(
    output: &mut String,
    character: char,
    previous_was_space: &mut bool,
    previous_was_digit_marker: &mut bool,
) {
    if character.is_ascii_digit() {
        if !*previous_was_digit_marker {
            output.push('#');
            *previous_was_digit_marker = true;
        }
        *previous_was_space = false;
    } else if character.is_whitespace() {
        if !*previous_was_space {
            output.push(' ');
            *previous_was_space = true;
        }
        *previous_was_digit_marker = false;
    } else {
        output.push(character);
        *previous_was_space = false;
        *previous_was_digit_marker = false;
    }
}
