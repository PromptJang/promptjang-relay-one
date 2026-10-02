//! An independent, old-version helper supervises startup and owns rollback.
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub parent_pid: u32,
    #[serde(default)]
    pub supervisor_pid: u32,
    #[serde(default)]
    pub supervisor_started: u64,
    pub application: PathBuf,
    pub executable: PathBuf,
    pub previous: PathBuf,
    pub data_dir: PathBuf,
    pub port: u16,
    pub version: String,
}

pub fn resume_if_needed(data_dir: &Path) -> Result<bool> {
    let path = data_dir.join("update-pending.json");
    if !path.exists() {
        return Ok(false);
    }
    let mut plan: RecoveryPlan = serde_json::from_slice(&fs::read(&path)?)?;
    validate_plan(&plan)?;
    if plan.supervisor_pid != 0
        && process_started(plan.supervisor_pid) == Some(plan.supervisor_started)
    {
        return Ok(false);
    }
    plan.parent_pid = std::process::id();
    fs::write(&path, serde_json::to_vec(&plan)?)?;
    let helper = data_dir.join("previous-update").join(if cfg!(windows) {
        "recovery.exe"
    } else {
        "recovery"
    });
    Command::new(helper)
        .arg("supervise-update")
        .arg("--plan")
        .arg(path)
        .spawn()?;
    Ok(true)
}

pub fn check_installation() -> Result<()> {
    #[cfg(target_os = "linux")]
    anyhow::ensure!(
        std::env::var_os("APPIMAGE").is_some(),
        "Use the package manager for deb/rpm installations; in-app Linux updates require AppImage"
    );
    let executable = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_exe()?)
        .canonicalize()?;
    let root = application_root(&executable);
    let parent = root.parent().context("installation parent")?;
    let probe = parent.join(format!(".pj-one-update-probe-{}", std::process::id()));
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .context("Installation is not writable; move the application to a writable directory")?;
    drop(file);
    fs::remove_file(probe)?;
    Ok(())
}

pub fn application_root(executable: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    for ancestor in executable.ancestors() {
        if ancestor.extension().is_some_and(|value| value == "app") {
            return ancestor.to_path_buf();
        }
    }
    // Windows installer updates a directory; AppImage is one file.
    #[cfg(target_os = "windows")]
    if let Some(parent) = executable.parent() {
        return parent.to_path_buf();
    }
    executable.to_path_buf()
}

pub fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(fs::read_link(source)?, destination)?;
            return Ok(());
        }
        #[cfg(not(unix))]
        {
            bail!("symbolic links are not supported in Windows update backups")
        }
    }
    if metadata.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else {
        fs::copy(source, destination)?;
    }
    fs::set_permissions(destination, metadata.permissions())?;
    Ok(())
}

pub fn prepare(data_dir: &Path, port: u16, version: String) -> Result<(PathBuf, PathBuf)> {
    let data_dir = data_dir.canonicalize()?;
    let executable = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_exe()?)
        .canonicalize()?;
    let application = application_root(&executable);
    anyhow::ensure!(
        !data_dir.starts_with(&application),
        "Application data must be outside the installation directory for safe rollback"
    );
    let backup = data_dir.join("previous-update");
    if backup.exists() {
        // Never overwrite unresolved recovery evidence.
        if data_dir.join("update-pending.json").exists() {
            bail!("an update is already pending")
        }
        fs::remove_dir_all(&backup).context("remove the previous completed update backup")?;
    }
    fs::create_dir(&backup)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&backup, fs::Permissions::from_mode(0o700))?;
    }
    let previous = backup.join("application");
    copy_tree(&application, &previous)?;
    // Service must be closed and SQLite checkpointed before this function.
    fs::copy(data_dir.join("relay-one.db"), backup.join("relay-one.db"))?;
    let helper = backup.join(if cfg!(windows) {
        "recovery.exe"
    } else {
        "recovery"
    });
    fs::copy(&executable, &helper)?;
    let plan = RecoveryPlan {
        parent_pid: std::process::id(),
        supervisor_pid: 0,
        supervisor_started: 0,
        application,
        executable,
        previous,
        data_dir: data_dir.to_path_buf(),
        port,
        version,
    };
    let path = data_dir.join("update-pending.json");
    fs::write(&path, serde_json::to_vec(&plan)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    Ok((helper, path))
}

fn alive(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), true);
    system.process(Pid::from_u32(pid)).is_some()
}

fn process_started(pid: u32) -> Option<u64> {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), true);
    system
        .process(Pid::from_u32(pid))
        .map(|process| process.start_time())
}

fn validate_plan(plan: &RecoveryPlan) -> Result<()> {
    anyhow::ensure!(
        plan.data_dir.is_absolute() && plan.executable.is_absolute(),
        "Recovery paths must be absolute"
    );
    anyhow::ensure!(
        plan.previous == plan.data_dir.join("previous-update/application"),
        "Unexpected recovery backup path"
    );
    anyhow::ensure!(
        plan.application == application_root(&plan.executable),
        "Recovery target is not the recorded application"
    );
    anyhow::ensure!(
        !plan.data_dir.starts_with(&plan.application),
        "Recovery data cannot be inside the application"
    );
    anyhow::ensure!(
        plan.previous.exists(),
        "Recovery application backup is missing"
    );
    Ok(())
}

pub fn supervise(path: &Path) -> Result<()> {
    let lock_path = path
        .parent()
        .context("recovery directory")?
        .join("update-recovery.lock");
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;
    if lock.try_lock().is_err() {
        return Ok(());
    }
    let mut plan: RecoveryPlan = serde_json::from_slice(&fs::read(path)?)?;
    validate_plan(&plan)?;
    plan.supervisor_pid = std::process::id();
    plan.supervisor_started =
        process_started(plan.supervisor_pid).context("identify recovery process")?;
    fs::write(path, serde_json::to_vec(&plan)?)?;
    for _ in 0..120 {
        if !path.exists() {
            return Ok(());
        } // Installation cancelled.
        if !alive(plan.parent_pid) {
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    if alive(plan.parent_pid) {
        bail!("update parent did not exit; backup retained")
    }
    #[cfg(target_os = "windows")]
    {
        let installer = plan
            .previous
            .parent()
            .context("backup directory")?
            .join("update-installer.exe");
        let status = Command::new(installer)
            .arg("/S")
            .arg("/UPDATE")
            .arg(format!("/D={}", plan.application.display()))
            .status()?;
        if !status.success() {
            restore(&plan)?;
            fs::remove_file(path)?;
            Command::new(&plan.executable)
                .arg("--data-dir")
                .arg(&plan.data_dir)
                .arg("--port")
                .arg(plan.port.to_string())
                .spawn()?;
            bail!("update installer failed; previous application restored");
        }
    }
    let launched = Command::new(&plan.executable)
        .arg("--data-dir")
        .arg(&plan.data_dir)
        .arg("--port")
        .arg(plan.port.to_string())
        .spawn();
    let mut child = match launched {
        Ok(child) => child,
        Err(_) => {
            restore(&plan)?;
            fs::remove_file(path)?;
            Command::new(&plan.executable)
                .arg("--data-dir")
                .arg(&plan.data_dir)
                .arg("--port")
                .arg(plan.port.to_string())
                .spawn()?;
            return Ok(());
        }
    };
    let ready = tokio::runtime::Runtime::new()?.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;
        for _ in 0..30 {
            let base = format!("http://127.0.0.1:{}", plan.port);
            if let Ok(response) = client.get(format!("{base}/api/v1/system")).send().await
                && response.status().is_success()
                && let Ok(value) = response.json::<serde_json::Value>().await
                && value["version"] == plan.version
                && value["database_path"].as_str() == plan.data_dir.join("relay-one.db").to_str()
                && let Ok(response) = client.get(format!("{base}/ready")).send().await
                && response.status().is_success()
            {
                return Ok::<bool, anyhow::Error>(true);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Ok(false)
    })?;
    if ready {
        fs::remove_file(path)?;
        fs::write(
            plan.data_dir.join("last-update.json"),
            serde_json::to_vec(&serde_json::json!({"version":plan.version,"outcome":"installed"}))?,
        )?;
        return Ok(());
    }
    let _ = child.kill();
    let _ = child.wait();
    // Do not restore while any new process could still be writing.
    if alive(child.id()) {
        bail!("updated process still alive; automatic restore refused")
    }
    restore(&plan)?;
    fs::remove_file(path)?;
    fs::write(
        plan.data_dir.join("last-update.json"),
        serde_json::to_vec(&serde_json::json!({"version":plan.version,"outcome":"rolled-back"}))?,
    )?;
    Command::new(&plan.executable)
        .arg("--data-dir")
        .arg(&plan.data_dir)
        .arg("--port")
        .arg(plan.port.to_string())
        .spawn()?;
    Ok(())
}

fn restore(plan: &RecoveryPlan) -> Result<()> {
    if plan.application.is_dir() {
        fs::remove_dir_all(&plan.application)?;
    } else if plan.application.exists() {
        fs::remove_file(&plan.application)?;
    }
    copy_tree(&plan.previous, &plan.application)?;
    for suffix in ["-wal", "-shm"] {
        let sidecar = plan.data_dir.join(format!("relay-one.db{suffix}"));
        if sidecar.exists() {
            fs::remove_file(sidecar)?;
        }
    }
    fs::copy(
        plan.previous
            .parent()
            .context("backup parent")?
            .join("relay-one.db"),
        plan.data_dir.join("relay-one.db"),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restores_application_and_database_together() {
        let root = std::env::temp_dir().join(format!("pj-update-{}", std::process::id()));
        fs::create_dir_all(root.join("backup/application")).unwrap();
        fs::create_dir_all(root.join("app")).unwrap();
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(root.join("backup/application/bin"), "old").unwrap();
        fs::write(root.join("backup/relay-one.db"), "old-db").unwrap();
        fs::write(root.join("app/bin"), "new").unwrap();
        fs::write(root.join("data/relay-one.db"), "new-db").unwrap();
        fs::write(root.join("data/relay-one.db-wal"), "pending").unwrap();
        let plan = RecoveryPlan {
            parent_pid: 0,
            supervisor_pid: 0,
            supervisor_started: 0,
            application: root.join("app"),
            executable: root.join("app/bin"),
            previous: root.join("backup/application"),
            data_dir: root.join("data"),
            port: 8081,
            version: "0.4.0".into(),
        };
        restore(&plan).unwrap();
        assert_eq!(fs::read(root.join("app/bin")).unwrap(), b"old");
        assert_eq!(fs::read(root.join("data/relay-one.db")).unwrap(), b"old-db");
        assert!(!root.join("data/relay-one.db-wal").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
