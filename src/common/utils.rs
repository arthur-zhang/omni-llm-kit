use std::ops::AddAssign;

pub fn post_inc<T: From<u8> + AddAssign<T> + Copy>(value: &mut T) -> T {
    let prev = *value;
    *value += T::from(1);
    prev
}


pub fn truncate_lines_to_byte_limit(s: &str, max_bytes: usize) -> &str {
    if s.len() < max_bytes {
        return s;
    }

    for i in (0..max_bytes).rev() {
        if s.is_char_boundary(i) {
            if s.as_bytes()[i] == b'\n' {
                // Since the i-th character is \n, valid to slice at i + 1.
                return &s[..i + 1];
            }
        }
    }

    truncate_to_byte_limit(s, max_bytes)
}

/// Truncates the string at a character boundary, such that the result is less than `max_bytes` in
/// length.
pub fn truncate_to_byte_limit(s: &str, max_bytes: usize) -> &str {
    if s.len() < max_bytes {
        return s;
    }

    for i in (0..max_bytes).rev() {
        if s.is_char_boundary(i) {
            return &s[..i];
        }
    }

    ""
}