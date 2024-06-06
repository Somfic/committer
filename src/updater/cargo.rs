use std::fs;

pub fn set_version(version: &semver::Version) -> anyhow::Result<()> {
    // TODO: Only update Cargo.toml files that are not git ignored

    // Find all Cargo.toml files in the repository
    let cargo_files = fs::read_dir(".")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains("Cargo.toml"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    // Update the version in all Cargo.toml files
    for file in cargo_files {
        let content = fs::read_to_string(&file)?;
        let regex = regex::Regex::new(r#"version\s*=\s*".+""#)?;
        let updated_content = regex
            .replace(&content, format!("version = \"{}\"", version))
            .to_string();

        fs::write(&file, updated_content)?;
    }

    Ok(())
}
