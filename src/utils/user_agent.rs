pub fn user_agent() -> String {
    format!(
        "TU-graxil {}({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    )
}
