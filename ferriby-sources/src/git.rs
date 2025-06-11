pub fn check_files() -> bool {
    std::fs::exists("./signal").unwrap_or(false)
}
