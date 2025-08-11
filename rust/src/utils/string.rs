pub fn outside<'a>(str: &'a str, start: &'a str, end: &'a str) -> Option<(&'a str, &'a str)> {
    if let Some(start_index) = str.find(start) {
        let start_end = start_index + start.len();
        if let Some(end_offset) = str[start_end..].find(end) {
            return Some((&str[..start_end], &str[(start_end + end_offset)..]));
        }
    }

    None
}

pub fn add_user_config<'a>(
    str: &'a str,
    user_config_start: &'a str,
    insert: &'a str,
) -> Option<String> {
    if let Some(start_index) = str.find(user_config_start) {
        let start_end = start_index + user_config_start.len();
        return Some(str[..start_end].to_owned() + insert + &str[start_end..]);
    }

    None
}
