use atty::Stream;

pub fn use_color() -> bool {
    atty::is(Stream::Stderr) && std::env::var_os("NO_COLOR").is_none()
}

pub fn color(code: &str, text: &str) -> String {
    if use_color() {
        format!("\x1b[{}m{}\x1b[0m", code, text)
    } else {
        text.to_string()
    }
}

pub fn c_prefix(text: &str) -> String {
    // 软绿
    color("1;32", text)
}

pub fn c_accent(text: &str) -> String {
    // 亮青
    color("1;36", text)
}
