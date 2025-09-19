use std::env;
use std::io;
use std::path::PathBuf;

use winres::WindowsResource;

use crate::uppercase_first;

/// Template app manifest for windows app.
const MANIFEST_TEMPLATE: &str = include_str!("../assets/app.manifest");
/// Template app admin manifest for windows app.
const MANIFEST_ADMIN_TEMPLATE: &str = include_str!("../assets/app_admin.manifest");
/// Template batch script for windows app.
const BATCH_TEMPLATE: &str = include_str!("../assets/build_windows.bat");

/// Configures a windows app build.
pub fn configure<I: Into<String>>(icon: I, admin: bool) -> io::Result<()> {
    WindowsResource::new()
        .set_manifest(if admin {
            MANIFEST_ADMIN_TEMPLATE
        } else {
            MANIFEST_TEMPLATE
        })
        .set_icon(&icon.into())
        .compile()?;

    let name = env::var("CARGO_PKG_NAME").unwrap();
    let name_uppercase = uppercase_first(&name);

    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    std::fs::create_dir_all(project.join("releases"))?;

    let exe_path = project
        .join("target")
        .join("release")
        .join(&name)
        .with_extension("exe")
        .display()
        .to_string();

    let target_exe_path = project
        .join("releases")
        .join(&name)
        .with_extension("exe")
        .display()
        .to_string();

    let releases = project.join("releases").display().to_string();

    let zip_path = project
        .join("releases")
        .join(format!("{}-v{}.zip", name_uppercase, version))
        .display()
        .to_string();

    let build_windows = BATCH_TEMPLATE
        .replace("{EXE-PATH}", &exe_path)
        .replace("{TARGET-EXE-PATH}", &target_exe_path)
        .replace("{RELEASES}", &releases)
        .replace("{ZIP-PATH}", &zip_path);

    std::fs::write(
        project
            .join("releases")
            .join("build_windows")
            .with_extension("bat"),
        build_windows,
    )?;

    Ok(())
}
