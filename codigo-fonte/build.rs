fn main() {
    #[cfg(windows)]
    {}
    if std::env::var("CARGO_CFG_TARGET_OS").map(|o| o == "windows").unwrap_or(false) {
        embed_resource::compile("icon.rc", embed_resource::NONE);
    }
}

