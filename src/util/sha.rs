// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

/// Gets the SHA256 digest of a file, given its path.
pub fn get_digest(path: PathBuf) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
