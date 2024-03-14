pub fn string_to_bool(value: String) -> bool {
    let lower_value = value.to_lowercase();
    match lower_value.as_str() {
        "true" => true,
        "false" => false,
        _ => false,
    }
}
