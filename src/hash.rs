use std::collections::HashMap;

use parking_lot::RwLock;
use prc::hash40::{Hash40, to_hash40};



lazy_static::lazy_static! {
    static ref HASHES: RwLock<HashMap<Hash40, String>> = RwLock::new(HashMap::new());
}

pub fn add_hash<S: AsRef<str>>(string: S) {
    let s = string.as_ref();
    HASHES.write().insert(to_hash40(s), s.to_string());
}

pub fn add_hashes(strings: Vec<&str>) {
    let mut hashes = HASHES.write();
    for s in strings {
        hashes.insert(to_hash40(s), s.to_string());
    }
}

pub fn try_get(hash: Hash40) -> Option<String> {
    HASHES.read().get(&hash).map(|x| x.clone())
}

pub fn get(hash: Hash40) -> String {
    match try_get(hash) {
        Some(s) => s.to_string(),
        None => format!("{:#x}", hash.0)
    }
}