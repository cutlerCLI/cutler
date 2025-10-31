use anyhow::{Result, bail};
use tokio::process::Command;

/// Represents an app installed from the Apple App Store.
///
/// The full list is fetched from mas and contains the first two properties;
/// - id: The identifier for the app.
/// - name: The name for the app.
#[derive(Debug)]
pub struct MasApplication {
    pub id: String,
    pub name: String,
}

/// Returns a list of MasApplication struct instances.
pub async fn list_apps() -> Result<Vec<MasApplication>> {
    if which::which("mas").is_err() {
        bail!("mas was not found in $PATH, so cannot check for installed apps.");
    }

    let output = Command::new("mas").arg("list").output().await?;

    if !output.status.success() {
        bail!("Could not fetch app list from mas.");
    }

    let list = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ' ');

            let id = parts.next()?.to_string();
            let name = parts
                .next()?
                .split_whitespace()
                .collect::<Vec<_>>()
                .split_last()
                .map(|(_, rest)| rest.join(" "))
                .unwrap_or_default();

            Some(MasApplication { id, name })
        })
        .collect();

    Ok(list)
}
