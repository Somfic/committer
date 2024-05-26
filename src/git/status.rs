use crate::cmd::execute;

pub struct Status {
    pub branch: String,
    pub commits_behind: u32,
    pub commits_ahead: u32,
}

pub fn status() -> Status {
    execute("git remote update").unwrap();
    let status = execute("git --no-pager status -s -b --porcelain").unwrap();

    let regex = regex::Regex::new(
        r"## (.+?)(?:\.{3})?(?:\s\[(?:ahead (\d+))?(?:, )?(?:behind (\d+))?\])?$",
    )
    .unwrap();

    let captures = regex.captures(&status).unwrap();
    let branch = captures.get(1).unwrap().as_str().to_string();
    let commits_ahead = captures.get(2).map_or(0, |m| m.as_str().parse().unwrap());
    let commits_behind = captures.get(3).map_or(0, |m| m.as_str().parse().unwrap());

    Status {
        branch,
        commits_ahead,
        commits_behind,
    }
}
