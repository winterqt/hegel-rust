//! Build script that ensures the hegel binary is available.
//!
//! This script:
//! 1. Checks if hegel is already on PATH
//! 2. If not, downloads uv and uses it to install hegel into a local venv
//! 3. Exports the path via HEGEL_BINARY_PATH environment variable

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Only re-run if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    let hegel_path = ensure_hegel();

    // Expose path to main crate via env var
    println!("cargo:rustc-env=HEGEL_BINARY_PATH={}", hegel_path.display());
}

/// Ensure hegel is available, installing it if necessary.
/// Returns the path to the hegel binary.
fn ensure_hegel() -> PathBuf {
    // 1. Check if hegel is already on PATH
    if let Some(path) = find_on_path("hegel") {
        eprintln!("cargo:warning=Found hegel on PATH: {}", path.display());
        return path;
    }

    // 2. Check if already installed in OUT_DIR cache
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cache_dir = out_dir.join("hegel-install");
    let venv_dir = cache_dir.join("venv");
    let hegel_bin = venv_dir.join("bin").join("hegel");

    if hegel_bin.exists() {
        eprintln!("cargo:warning=Using cached hegel: {}", hegel_bin.display());
        return hegel_bin;
    }

    // 3. Install from scratch
    eprintln!("cargo:warning=Installing hegel...");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");

    let uv_path = ensure_uv(&cache_dir);

    // Create venv with Python 3.13
    eprintln!("cargo:warning=Creating Python venv...");
    let status = Command::new(&uv_path)
        .args(["venv", "--python", "3.13"])
        .arg(&venv_dir)
        .status()
        .expect("Failed to run uv venv");

    if !status.success() {
        panic!("Failed to create venv (exit code: {:?})", status.code());
    }

    // Install hegel from git
    eprintln!("cargo:warning=Installing hegel from git...");
    let status = Command::new(&uv_path)
        .args([
            "pip",
            "install",
            "git+ssh://git@github.com/antithesishq/hegel.git",
        ])
        .arg("--python")
        .arg(venv_dir.join("bin").join("python"))
        .status()
        .expect("Failed to run uv pip install");

    if !status.success() {
        panic!(
            "Failed to install hegel (exit code: {:?}). \
             Make sure you have SSH access to github.com/antithesishq/hegel",
            status.code()
        );
    }

    if !hegel_bin.exists() {
        panic!(
            "hegel binary not found at {} after installation",
            hegel_bin.display()
        );
    }

    eprintln!("cargo:warning=Installed hegel: {}", hegel_bin.display());
    hegel_bin
}

/// Ensure uv is available, downloading it if necessary.
/// Returns the path to the uv binary.
fn ensure_uv(cache_dir: &Path) -> PathBuf {
    // Check if uv already on PATH
    if let Some(path) = find_on_path("uv") {
        eprintln!("cargo:warning=Found uv on PATH: {}", path.display());
        return path;
    }

    let uv_dir = cache_dir.join("uv");
    let uv_bin = uv_dir.join("uv");

    // Check cache
    if uv_bin.exists() {
        eprintln!("cargo:warning=Using cached uv: {}", uv_bin.display());
        return uv_bin;
    }

    // Download uv
    eprintln!("cargo:warning=Downloading uv...");
    fs::create_dir_all(&uv_dir).expect("Failed to create uv dir");

    let status = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "curl -LsSf https://astral.sh/uv/install.sh | UV_INSTALL_DIR={} sh",
            uv_dir.display()
        ))
        .status()
        .expect("Failed to run curl for uv install script");

    if !status.success() {
        panic!("Failed to install uv (exit code: {:?})", status.code());
    }

    if !uv_bin.exists() {
        panic!(
            "uv binary not found at {} after installation",
            uv_bin.display()
        );
    }

    eprintln!("cargo:warning=Installed uv: {}", uv_bin.display());
    uv_bin
}

/// Find an executable on PATH.
fn find_on_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(name);
                if full_path.is_file() && is_executable(&full_path) {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

/// Check if a file is executable (Unix only).
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &PathBuf) -> bool {
    true // On Windows, just check existence
}
