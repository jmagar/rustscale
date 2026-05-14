use std::{fs, process::Command};

fn tailscale_bin() -> &'static str {
    env!("CARGO_BIN_EXE_tailscale")
}

fn make_fake_binary(dir: &std::path::Path) {
    let path = dir.join("tailscale");
    fs::write(&path, "#!/usr/bin/env sh\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    fs::set_permissions(path, perms).unwrap();
}

fn make_capture_binary(dir: &std::path::Path, name: &str, env_key: &str) {
    let path = dir.join(name);
    fs::write(
        &path,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$*\" > \"$CAPTURE_FILE\"\nprintf '%s\\n' \"${{{env_key}:-}}\" > \"$CAPTURE_ENV_FILE\"\n"
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    fs::set_permissions(path, perms).unwrap();
}

#[test]
fn setup_plugin_hook_no_repair_json_contract() {
    let home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    make_fake_binary(bin_dir.path());

    let output = Command::new(tailscale_bin())
        .args(["--json", "setup", "plugin-hook", "--no-repair"])
        .env("TAILSCALE_MCP_HOME", home.path().join(".tailscale-test"))
        .env("PATH", bin_dir.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "no-repair should report missing appdata/env as blocking"
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["exit_policy"], "blocking_failure");
    assert_eq!(payload["ran_repair"], false);
    assert_eq!(payload["no_repair"], true);
    assert!(payload["blocking_failures"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "appdata_dir"));
    assert!(payload["advisory_failures"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "env_file"));
    assert!(!home.path().join(".tailscale-test").exists());
}

#[test]
fn plugin_hook_adapter_delegates_to_binary() {
    let home = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_file = home.path().join("args.txt");
    let capture_env_file = home.path().join("env.txt");
    make_capture_binary(bin_dir.path(), "tailscale", "TAILSCALE_MCP_TOKEN");

    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("plugins/tailscale/hooks/plugin-setup.sh");
    let path = format!(
        "{}:{}",
        bin_dir.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(script)
        .env("HOME", home.path())
        .env("PATH", path)
        .env("CLAUDE_PLUGIN_DATA", data.path())
        .env("CAPTURE_FILE", &capture_file)
        .env("CAPTURE_ENV_FILE", &capture_env_file)
        .env("CLAUDE_PLUGIN_OPTION_API_TOKEN", "test-token")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(capture_file).unwrap().trim(),
        "setup plugin-hook"
    );
    assert_eq!(
        fs::read_to_string(capture_env_file).unwrap().trim(),
        "test-token"
    );
    assert!(data.path().is_dir());
}
