//! Locating the matching installed viewer and starting it without a shell.

fn viewer() -> Option<(&'static str, &'static str)> {
    match solidcraft_engine::product::PRODUCT.name {
        "HasiSolid" => Some(("Hasi 3D", "hasi-3d.exe")),
        "MSSolid" => Some(("MS3D", "ms3d.exe")),
        _ => None,
    }
}

#[cfg(windows)]
pub fn find() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
    let (display, exe) = viewer()?;
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
            let Ok(parent) = RegKey::predef(root).open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Uninstall", KEY_READ | view)
            else {
                continue;
            };
            for key in parent.enum_keys().flatten() {
                let Ok(entry) = parent.open_subkey_with_flags(key, KEY_READ | view) else { continue };
                let name: String = entry.get_value("DisplayName").unwrap_or_default();
                if !name.eq_ignore_ascii_case(display) {
                    continue;
                }
                let location: String = entry.get_value("InstallLocation").unwrap_or_default();
                let candidate = std::path::Path::new(location.trim().trim_matches('"')).join(exe);
                if candidate.is_file() {
                    return Some(candidate.to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn find() -> Option<String> {
    None
}

pub fn launch(executable: &str, step: &str) -> Result<(), String> {
    let (_, expected) = viewer().ok_or("No viewer is assigned to this product")?;
    let exe = std::path::Path::new(executable);
    if !exe.is_file() || !exe.file_name().is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(expected)) {
        return Err(format!("Select {expected}"));
    }
    if !std::path::Path::new(step).is_file() {
        return Err("The STEP file is not available".into());
    }
    let mut command = std::process::Command::new(exe);
    command.arg(step);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command.spawn().map(|_| ()).map_err(|error| error.to_string())
}
