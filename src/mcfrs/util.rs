pub fn get_indent(line: &str) -> usize {
    match line.chars()
        .position(|c| c != '\t') {
            Some(i) => i + 1,
            None => 0,
    }
}
