use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_iso_string() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    format!("{now}")
}

pub fn unique_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    format!("{prefix}_{nanos}")
}

pub fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => ch,
            _ => '_',
        })
        .collect()
}

pub fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

pub fn int_to_bool(value: i64) -> bool {
    value != 0
}
