use anyhow::Ok;

pub fn calculate_new_tag_based_on_commits() -> anyhow::Result<String> {
    let latest_tag = crate::git::tag::latest()?;

    // Clean up tag by adding patch version if it's missing
    let latest_tag_clean = if latest_tag.to_string().split('.').count() == 2 {
        format!("{}.0", latest_tag)
    } else {
        latest_tag.to_string()
    };

    let latest_version = semver::Version::parse(&latest_tag_clean)?;

    let patches_delta = crate::git::log::patches_since(&latest_tag.to_string())?;
    let minors_delta = crate::git::log::minors_since(&latest_tag.to_string())?;
    let majors_delta = crate::git::log::majors_since(&latest_tag.to_string())?;

    let mut patches = latest_version.patch;
    let mut minors = latest_version.minor;
    let mut majors = latest_version.major;

    if majors_delta.len() > 0 {
        majors += 1;
        minors = 0;
        patches = 0;
    } else if minors_delta.len() > 0 {
        minors += 1;
        patches = 0;
    } else if patches_delta.len() > 0 {
        patches += 1;
    }

    let new_version = semver::Version::new(majors, minors, patches);

    if new_version == latest_version {
        println!("No new commits since the latest tag.");
    }

    Ok(new_version.to_string())
}
