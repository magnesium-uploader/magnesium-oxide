#![allow(dead_code)]

fn get_time() -> String {
    chrono::offset::Utc::now().to_rfc2822()
}

pub fn debug<T: std::fmt::Display>(t: T) {
    println!("\x1b[90m[{}] DEBUG: {}\x1b[0m", get_time(), t);
}

pub fn info<T: std::fmt::Display>(t: T) {
    println!("\x1b[1;34m[{}] INFO: {}\x1b[0m", get_time(), t);
}

pub fn warn<T: std::fmt::Display>(t: T) {
    println!("\x1b[1;33m[{}] WARN: {}\x1b[0m", get_time(), t);
}

pub fn error<T: std::fmt::Display>(t: T) {
    println!("\x1b[1;31m[{}] ERROR: {}\x1b[0m", get_time(), t);
}

pub fn critical<T: std::fmt::Display>(t: T) {
    println!("\x1b[1;31m[{}] CRITICAL: {}\x1b[0m", get_time(), t);
}