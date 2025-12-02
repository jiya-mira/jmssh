use atty::Stream;

pub fn use_color() -> bool {
    atty::is(Stream::Stderr) && std::env::var_os("NO_COLOR").is_none()
}

pub fn color(code: &str, text: &str) -> String {
    if use_color() {
        format!("\x1b[{}m{}\x1b[0m", code, text.to_ascii_lowercase())
    } else {
        text.to_ascii_lowercase().to_string()
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

pub fn c_error(text: &str) -> String {
    // 红
    color("1;31", text)
}

pub fn log_info(msg: impl AsRef<str>) {
    eprintln!("{} {}", c_prefix("[jmssh]"), msg.as_ref());
}

pub fn log_warn(msg: impl AsRef<str>) {
    // 这里也不强行上黄色，正文颜色交给调用方
    eprintln!("{} {}", c_prefix("[jmssh]"), msg.as_ref());
}

pub fn log_error(msg: impl AsRef<str>) {
    // 同理，不全行上红，只是用相同前缀
    eprintln!("{} {}", c_prefix("[jmssh]"), msg.as_ref());
}
