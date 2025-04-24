use crate::{
    config::loader::load_config,
    defaults::{executor, flags},
    domains::collector,
    external::runner,
    snapshot::state::{SettingState, Snapshot},
    util::io::confirm_action,
    util::logging::{LogLevel, print_log},
};
use anyhow::Result;

pub fn run(verbose: bool, dry_run: bool) -> Result<()> {
    let config_path = crate::config::loader::get_config_path();
    if !config_path.exists() {
        print_log(
            LogLevel::Info,
            &format!("Config not found at {:?}", config_path),
        );
        if confirm_action("Create new config?")? {
            super::init::run(verbose, false)?;
            print_log(
                LogLevel::Info,
                "Config created; edit it and then `cutler apply`.",
            );
            return Ok(());
        } else {
            anyhow::bail!("No config; aborting.");
        }
    }

    // parse + flatten domains
    let toml = load_config(&config_path)?;
    let domains = collector::collect(&toml)?;

    // load snapshot if any
    let snap_path = crate::snapshot::state::get_snapshot_path();
    let snapshot = if snap_path.exists() {
        match Snapshot::load(&snap_path) {
            Ok(s) => s,
            Err(e) => {
                print_log(LogLevel::Warning, &format!("Bad snapshot: {}; new one.", e));
                Snapshot::new()
            }
        }
    } else {
        Snapshot::new()
    };

    // build a map of existing snapshot entries
    let mut existing = snapshot
        .settings
        .into_iter()
        .map(|s| ((s.domain.clone(), s.key.clone()), s))
        .collect::<std::collections::HashMap<_, _>>();
    let mut seen = std::collections::HashSet::new();

    // walk every (domain, key, value) => compare, write, update snapshot map
    for (dom, tbl) in domains {
        if collector::needs_prefix(&dom) {
            collector::check_exists(&format!("com.apple.{}", dom))?;
        }
        for (key, val) in tbl {
            let (eff_dom, eff_key) = collector::effective(&dom, &key);
            let desired = flags::normalize(&val);

            seen.insert((eff_dom.clone(), eff_key.clone()));
            let current = collector::read_current(&eff_dom, &eff_key);
            let entry = existing.remove(&(eff_dom.clone(), eff_key.clone()));

            match entry {
                Some(mut old) if old.new_value != desired => {
                    let (flag, val_str) = flags::to_flag(&val)?;
                    executor::write(
                        &eff_dom, &eff_key, flag, &val_str, "Updating", verbose, dry_run,
                    )?;
                    old.new_value = desired.clone();
                    existing.insert((eff_dom.clone(), eff_key.clone()), old);
                }
                None => {
                    // new or never applied
                    if current.as_ref() != Some(&desired) {
                        let (flag, val_str) = flags::to_flag(&val)?;
                        executor::write(
                            &eff_dom, &eff_key, flag, &val_str, "Applying", verbose, dry_run,
                        )?;
                    }
                    let initial = SettingState {
                        domain: eff_dom.clone(),
                        key: eff_key.clone(),
                        original_value: current.clone(),
                        new_value: desired.clone(),
                    };
                    existing.insert((eff_dom.clone(), eff_key.clone()), initial);
                }
                Some(old) => {
                    if verbose {
                        print_log(
                            LogLevel::Info,
                            &format!("Skipping unchanged {}.{}", eff_dom, eff_key),
                        );
                    }
                    // put it back
                    existing.insert((eff_dom.clone(), eff_key.clone()), old);
                }
            }
        }
    }

    let mut final_snap = Snapshot::new();
    final_snap.settings = existing.into_iter().map(|(_, s)| s).collect();
    final_snap.external = runner::extract(&toml);

    if !dry_run {
        final_snap.save(&snap_path)?;
        if verbose {
            print_log(
                LogLevel::Success,
                &format!("Snapshot saved: {:?}", snap_path),
            );
        }
    } else {
        print_log(LogLevel::Info, "Dry-run: would save snapshot");
    }

    // exec external commands
    let _ = runner::run_all(&toml, verbose, dry_run);

    Ok(())
}
