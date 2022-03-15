#[allow(dead_code)]
fn get_time() -> String {
    chrono::offset::Utc::now().to_rfc2822()
}

#[allow(dead_code)]
pub fn debug(message: &str) {
    println!("\x1b[90m[{}] DEBUG: {}\x1b[0m", get_time(), message);
}

#[allow(dead_code)]
pub fn info(message: &str) {
    println!("\x1b[1;34m[{}] INFO: {}\x1b[0m", get_time(), message);
}

#[allow(dead_code)]
pub fn warn(message: &str) {
    println!("\x1b[1;33m[{}] WARN: {}\x1b[0m", get_time(), message);
}

#[allow(dead_code)]
pub fn error(message: &str) {
    println!("\x1b[1;31m[{}] ERROR: {}\x1b[0m", get_time(), message);
}

#[allow(dead_code)]
pub fn critical(message: &str) {
    println!("\x1b[1;35m[{}] CRITICAL: {}\x1b[0m", get_time(), message);
}