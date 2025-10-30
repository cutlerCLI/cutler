use anyhow::{Result, bail};
use tokio::process::Command;

pub struct AppStoreApplication {
    pub id: String,
    pub name: String,
}

pub async fn get_appstore_pkgs() -> Result<Vec<AppStoreApplication>> {
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
            let name = parts.next()?.trim().to_string();
            Some(AppStoreApplication { id, name })
        })
        .collect();

    Ok(list)
}
