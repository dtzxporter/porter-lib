use std::env;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::uppercase_first;

/// Template desktop file for linux app.
const DESKTOP_TEMPLATE: &str = include_str!("../assets/app.desktop");
/// Template bash script for linux app.
const BASH_TEMPLATE: &str = include_str!("../assets/build_linux.sh");

/// Configures a linux app build.
pub fn configure<I: Into<String>>(icon: I) -> io::Result<()> {
    let name = env::var("CARGO_PKG_NAME").unwrap();
    let name_uppercase = uppercase_first(&name);

    let description = env::var("CARGO_PKG_DESCRIPTION").unwrap();
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    std::fs::create_dir_all(project.join("linux"))?;

    let desktop_file = DESKTOP_TEMPLATE
        .replace("{NAME}", &name)
        .replace("{NAME-UPPERCASE}", &name_uppercase)
        .replace("{DESCRIPTION}", &description);

    std::fs::write(
        project
            .join("linux")
            .join(&name_uppercase)
            .with_extension("desktop"),
        desktop_file,
    )?;

    std::fs::copy(
        icon.into(),
        project.join("linux").join(&name).with_extension("png"),
    )?;

    std::fs::create_dir_all(project.join("releases"))?;

    let build_linux = BASH_TEMPLATE
        .replace("{NAME}", &name)
        .replace("{NAME-UPPERCASE}", &name_uppercase)
        .replace("{VERSION}", &version);

    let mut build_script = std::fs::File::create(
        project
            .join("releases")
            .join("build_linux")
            .with_extension("sh"),
    )?;

    let metadata = build_script.metadata()?;
    let mut perms = metadata.permissions();

    // Make the script executable, since we're writing it instead of copying it.
    perms.set_mode(perms.mode() | 0x1);

    build_script.write_all(build_linux.as_bytes())?;
    build_script.set_permissions(perms)?;

    Ok(())
}
