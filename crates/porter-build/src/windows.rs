use std::env;
use std::io;
use std::path::PathBuf;

use winresource::WindowsResource;

use crate::uppercase_first;

/// Template app manifest for windows app.
const MANIFEST_TEMPLATE: &str = include_str!("../assets/app.manifest");
/// Template app admin manifest for windows app.
const MANIFEST_ADMIN_TEMPLATE: &str = include_str!("../assets/app_admin.manifest");
/// Template batch script for windows app.
const BATCH_TEMPLATE: &str = include_str!("../assets/build_windows.bat");

/// Configures a windows app build.
pub fn configure(icon: Option<String>, admin: bool, dll: bool) -> io::Result<()> {
    let name = env::var("CARGO_PKG_NAME").unwrap();
    let name_uppercase = uppercase_first(&name);

    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let project = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_toml = project.join("Cargo.toml");

    let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    while let Some(parent) = out_dir.parent() {
        if parent.ends_with("target") {
            out_dir.pop();
            break;
        }

        out_dir.pop();
    }

    if !out_dir.exists() {
        panic!("Failed to find target directory!");
    }

    let cargo_toml = std::fs::read_to_string(project_toml)?;
    let cargo_toml: toml::Table =
        toml::from_str(&cargo_toml).expect("Failed to parse Cargo.toml for build configuration!");

    let build_configuration = cargo_toml
        .get("package")
        .and_then(|toml| toml.get("metadata"))
        .and_then(|toml| toml.get("porter"))
        .and_then(|toml| toml.get("build"))
        .and_then(|toml| toml.as_table())
        .expect("Missing package.metadata.porter.build configuration!");

    let original_filename = build_configuration
        .get("original-filename")
        .and_then(|value| value.as_str())
        .expect("Invalid package.metadata.porter.build.original-filename configuration!");

    let file_description = build_configuration
        .get("file-description")
        .and_then(|value| value.as_str())
        .expect("Invalid package.metadata.porter.build.file-description configuration!");

    let legal_copyright = build_configuration
        .get("legal-copyright")
        .and_then(|value| value.as_str())
        .expect("Invalid package.metadata.porter.build.legal-copyright configuration!");

    let dll_name = build_configuration
        .get("dll-name")
        .and_then(|value| value.as_str());

    let mut resource = WindowsResource::new();

    resource
        .set("OriginalFilename", original_filename)
        .set("FileDescription", file_description)
        .set("LegalCopyright", legal_copyright)
        .set_manifest(if admin {
            MANIFEST_ADMIN_TEMPLATE
        } else {
            MANIFEST_TEMPLATE
        });

    if let Some(icon) = icon {
        resource.set_icon(&icon);
    }

    resource.compile()?;

    std::fs::create_dir_all(project.join("releases"))?;

    let exe_path = out_dir
        .join("release")
        .join(&name)
        .with_extension(if dll { "dll" } else { "exe" })
        .display()
        .to_string();

    let target_exe_path = project
        .join("releases")
        .join(if let Some(dll_name) = dll_name {
            dll_name
        } else {
            &name
        })
        .with_extension(if dll { "dll" } else { "exe" })
        .display()
        .to_string();

    let releases = project
        .join("releases")
        .display()
        .to_string();

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
