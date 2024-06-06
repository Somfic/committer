use anyhow::Ok;

pub fn calculate_new_tag_based_on_commits() -> anyhow::Result<Option<semver::Version>> {
    let latest_tag = crate::git::tag::latest()?;

    // Clean up tag by adding patch version if it's missing
    let latest_tag_clean = if latest_tag.split('.').count() == 2 {
        format!("{}.0", latest_tag)
    } else {
        latest_tag.to_string()
    }
    .replace("v", "");

    let latest_version = semver::Version::parse(&latest_tag_clean)?;

    let patches_delta = crate::git::log::patches_since(&latest_tag.to_string())?;
    let minors_delta = crate::git::log::minors_since(&latest_tag.to_string())?;
    let majors_delta = crate::git::log::majors_since(&latest_tag.to_string())?;

    if !majors_delta.is_empty() {
        println!("Major changes:");
        for commit in &majors_delta {
            println!("  - {}", commit.message);
        }
    }

    if !minors_delta.is_empty() {
        println!("Minor changes:");
        for commit in &minors_delta {
            println!("  - {}", commit.message);
        }
    }

    if !patches_delta.is_empty() {
        println!("Patch changes:");
        for commit in &patches_delta {
            println!("  - {}", commit.message);
        }
    }

    let mut patches = latest_version.patch;
    let mut minors = latest_version.minor;
    let mut majors = latest_version.major;

    if !majors_delta.is_empty() {
        majors += 1;
        minors = 0;
        patches = 0;
    } else if !minors_delta.is_empty() {
        minors += 1;
        patches = 0;
    } else if !patches_delta.is_empty() {
        patches += 1;
    }

    let new_version = semver::Version::new(majors, minors, patches);

    if new_version == latest_version {
        return Ok(None);
    }

    Ok(Some(new_version))
}
