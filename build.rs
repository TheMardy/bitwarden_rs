use std::process::Command;

fn main() {
    #[cfg(all(feature = "sqlite", feature = "mysql"))]
    compile_error!("Can't enable both backends");

    #[cfg(not(any(feature = "sqlite", feature = "mysql")))]
    compile_error!("You need to enable one DB backend. To build with previous defaults do: cargo build --features sqlite");

    read_git_info().ok();
}

fn run(args: &[&str]) -> Result<String, std::io::Error> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    if !out.status.success() {
        use std::io::{Error, ErrorKind};
        return Err(Error::new(ErrorKind::Other, "Command not successful"));
    }
    Ok(String::from_utf8(out.stdout).unwrap().trim().to_string())
}

/// This method reads info from Git, namely tags, branch, and revision
fn read_git_info() -> Result<(), std::io::Error> {
    // The exact tag for the current commit, can be empty when
    // the current commit doesn't have an associated tag
    let exact_tag = run(&["git", "describe", "--abbrev=0", "--tags", "--exact-match"]).ok();
    if let Some(ref exact) = exact_tag {
        println!("cargo:rustc-env=GIT_EXACT_TAG={}", exact);
    }

    // The last available tag, equal to exact_tag when
    // the current commit is tagged
    let last_tag = run(&["git", "describe", "--abbrev=0", "--tags"])?;
    println!("cargo:rustc-env=GIT_LAST_TAG={}", last_tag);

    // The current branch name
    let branch = run(&["git", "rev-parse", "--abbrev-ref", "HEAD"])?;
    println!("cargo:rustc-env=GIT_BRANCH={}", branch);

    // The current git commit hash
    let rev = run(&["git", "rev-parse", "HEAD"])?;
    let rev_short = rev.get(..8).unwrap_or_default();
    println!("cargo:rustc-env=GIT_REV={}", rev_short);

    // Combined version
    let version = if let Some(exact) = exact_tag {
        exact
    } else if &branch != "master" {
        format!("{}-{} ({})", last_tag, rev_short, branch)
    } else {
        format!("{}-{}", last_tag, rev_short)
    };
    println!("cargo:rustc-env=GIT_VERSION={}", version);

    // To access these values, use:
    //    env!("GIT_EXACT_TAG")
    //    env!("GIT_LAST_TAG")
    //    env!("GIT_BRANCH")
    //    env!("GIT_REV")
    //    env!("GIT_VERSION")

    Ok(())
}
