use std::backtrace::Backtrace;

use crate::system;

/// Encrypt/Decrypt an input with the given key.
fn xor_encrypt<K: AsRef<[u8]>>(input: String, key: K) -> Vec<u8> {
    let key = key.as_ref();
    let mut buffer = input.as_bytes().to_vec();

    for i in 0..buffer.len() {
        buffer[i] ^= key[i % key.len()];
    }

    buffer
}

/// Installs a global panic hook that will log encrypted information to a crash file.
pub fn install(name: &'static str, version: &'static str) {
    // Prevent panic hook from running on debug builds.
    if cfg!(debug_assertions) {
        return;
    }

    let target = system::config_dir()
        .join(name.to_lowercase())
        .with_extension("crash");

    let _ = std::fs::create_dir_all(system::config_dir());

    std::panic::set_hook(Box::new(move |error| {
        let backtrace = Backtrace::force_capture();
        let error = format!("{} {:?} ({})", error, backtrace, version);

        let _ = std::fs::write(target.clone(), xor_encrypt(error, "bijudama"));
    }));
}
