use std::env;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::uppercase_first;

/// Template plist for macOS app.
const PLIST_TEMPLATE: &str = include_str!("../assets/Info.plist");
/// Template bash script for macOS app.
const BASH_TEMPLATE: &str = include_str!("../assets/build_macos.sh");

/// Configures a macOS app build.
pub fn configure<I: Into<String>>(icon: I) -> io::Result<()> {
    let name = env::var("CARGO_PKG_NAME").unwrap();
    let name_uppercase = uppercase_first(&name);

    let description = env::var("CARGO_PKG_DESCRIPTION")
        .unwrap()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;");

    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    std::fs::create_dir_all(
        project
            .join("macOS")
            .join(format!("{}.app", name_uppercase))
            .join("Contents")
            .join("Resources"),
    )?;

    std::fs::copy(
        icon.into(),
        project
            .join("macOS")
            .join(format!("{}.app", name_uppercase))
            .join("Contents")
            .join("Resources")
            .join(name.as_str())
            .with_extension("icns"),
    )?;

    std::fs::create_dir_all(
        project
            .join("macOS")
            .join(format!("{}.app", name_uppercase))
            .join("Contents")
            .join("MacOS"),
    )?;

    let info_plist = PLIST_TEMPLATE
        .replace("{NAME}", &name)
        .replace("{NAME-UPPERCASE}", &name_uppercase)
        .replace("{VERSION}", &version)
        .replace("{DESCRIPTION}", &description);

    std::fs::write(
        project
            .join("macOS")
            .join(format!("{}.app", name_uppercase))
            .join("Contents")
            .join("Info")
            .with_extension("plist"),
        info_plist,
    )?;

    std::fs::create_dir_all(project.join("releases"))?;

    let build_macos = BASH_TEMPLATE
        .replace("{NAME}", &name)
        .replace("{NAME-UPPERCASE}", &name_uppercase)
        .replace("{VERSION}", &version);

    let mut build_script = std::fs::File::create(
        project
            .join("releases")
            .join("build_macos")
            .with_extension("sh"),
    )?;

    let metadata = build_script.metadata()?;
    let mut perms = metadata.permissions();

    // Make the script executable, since we're writing it instead of copying it.
    perms.set_mode(perms.mode() | 0x1);

    build_script.write_all(build_macos.as_bytes())?;
    build_script.set_permissions(perms)?;

    Ok(())
}
