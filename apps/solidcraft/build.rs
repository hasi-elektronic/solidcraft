#![forbid(unsafe_code)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hasi = std::env::var_os("CARGO_FEATURE_BRAND_HASISOLID").is_some();
    let ms = std::env::var_os("CARGO_FEATURE_BRAND_MSSOLID").is_some();
    if hasi && ms {
        return Err("Select only one product brand".into());
    }
    let (name, company, icon_path) = if hasi {
        ("HasiSolid", "Hasi Elektronic", "../../assets/brands/hasisolid/app.ico")
    } else if ms {
        ("MSSolid", "Manfred Sickinger GmbH", "../../assets/brands/mssolid/app.ico")
    } else {
        ("SolidCraft", "SolidCraft contributors", "../../assets/icons/app/windows/exports/solidcraft-executable.ico")
    };
    let icon = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(icon_path);
    println!("cargo:rerun-if-changed={}", icon.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    if std::env::var("CARGO_CFG_TARGET_OS")? != "windows" {
        return Ok(());
    }
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);
    let copy = out.join("solidcraft.ico");
    std::fs::copy(icon, &copy)?;
    let rc = out.join("solidcraft.rc");
    let icon_line = format!("1 ICON \"{}\"\n", copy.display().to_string().replace('\\', "/"));
    let version = if hasi || ms { "0.2.4".to_owned() } else { std::env::var("CARGO_PKG_VERSION")? };
    std::fs::write(&rc, format!("{icon_line}{}", version_resource(&version, name, company)))?;
    embed_resource::compile_for(&rc, ["solidcraft"], embed_resource::NONE).manifest_required()?;
    Ok(())
}

/// The Windows version resource (what Explorer's Details tab shows and what a launcher reads).
/// `FileVersion` and `ProductVersion` are the package version verbatim (`0.25.1`, `0.26.0-rc.1`):
/// ArtCraft's launcher parses the `ProductVersion` string as semver, so a fourth `.0` would not
/// read. The binary `FILEVERSION` fields need four numbers and take the leading `major.minor.patch`.
fn version_resource(version: &str, name: &str, company: &str) -> String {
    let mut numbers = version.split(['-', '+']).next().unwrap_or("").split('.').map(|p| p.parse::<u16>().unwrap_or(0));
    let mut next = || numbers.next().unwrap_or(0);
    let (major, minor, patch) = (next(), next(), next());
    let internal_name = if name == "SolidCraft" { "solidcraft" } else { name };
    let filename = format!("{internal_name}.exe");
    format!(
        "1 VERSIONINFO\nFILEVERSION {major},{minor},{patch},0\nPRODUCTVERSION {major},{minor},{patch},0\nFILEOS 0x40004\nFILETYPE 0x1\n{{\nBLOCK \"StringFileInfo\"\n{{\nBLOCK \"040904B0\"\n{{\nVALUE \"CompanyName\", \"{company}\"\nVALUE \"FileDescription\", \"{name}, parametric 3D CAD\"\nVALUE \"FileVersion\", \"{version}\"\nVALUE \"InternalName\", \"{internal_name}\"\nVALUE \"OriginalFilename\", \"{filename}\"\nVALUE \"ProductName\", \"{name}\"\nVALUE \"ProductVersion\", \"{version}\"\n}}\n}}\nBLOCK \"VarFileInfo\"\n{{\nVALUE \"Translation\", 0x0409, 1200\n}}\n}}\n"
    )
}
