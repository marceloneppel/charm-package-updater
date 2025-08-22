use clap::{arg, command, value_parser};
use git2::Repository;
use std::fs;
use std::path::Path;
use url::Url;

const TEMP_DIR: &str = "./temp";

fn main() {
    let matches = command!()
        .arg(
            arg!(
                --ppa <name> "PPA to use for the package"
            )
            .required(true)
            .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(
                --package <name> "Package that will be updated (format: name=version)"
            )
            .required(true)
            .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(
                --snaprepo <URL> "Snap repository URL to use for the Snap"
            )
            .required(true)
            .value_parser(value_parser!(Url)),
        )
        .arg(
            arg!(
                --snaprepobranch <branch> "Snap repository branch to use for the Snap"
            )
            .required(true)
            .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(
                --rockrepo <URL> "rock repository URL to use for the rock OCI"
            )
            .required(false)
            .value_parser(value_parser!(Url)),
        )
        .arg(
            arg!(
                --rockrepobranch <branch> "rock repository branch to use for the rock OCI"
            )
            .required(false)
            .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(
                --charmrepo <URL> "Charm repository URL to use for the charm"
            )
            .required(true)
            .value_parser(value_parser!(Url)),
        )
        .arg(
            arg!(
                --charmrepobranch <branch> "Charm repository branch to use for the charm"
            )
            .required(true)
            .value_parser(value_parser!(String)),
        )
        .get_matches();

    if Path::new(TEMP_DIR).is_dir() {
        match fs::remove_dir_all(TEMP_DIR) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error removing directory: {}", e);
                std::process::exit(1);
            }
        }
    }
    match fs::create_dir_all(TEMP_DIR) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error creating directory: {}", e);
            std::process::exit(1);
        }
    }

    if let Some(ppa) = matches.get_one::<String>("ppa") {
        println!("Value for PPA: {ppa}");
    }

    if let Some(package) = matches.get_one::<String>("package") {
        println!("Value for package: {package}");
    }

    for (repo_key, repo_branch_key) in [
        ("snaprepo", "snaprepobranch"),
        ("rockrepo", "rockrepobranch"),
        ("charmrepo", "charmrepobranch"),
    ] {
        if let Some(snap_repo) = matches.get_one::<Url>(repo_key) {
            let formatted_repo_name = snap_repo
                .as_str()
                .split('/')
                .last()
                .unwrap()
                .split('.')
                .next()
                .unwrap();
            let snap_repo_object = match Repository::clone(
                snap_repo.as_str(),
                Path::join(Path::new(TEMP_DIR), Path::new(formatted_repo_name)),
            ) {
                Ok(repo) => repo,
                Err(e) => {
                    eprintln!("failed to clone: {}", e);
                    std::process::exit(1);
                }
            };
            if let Some(snap_repo_branch) = matches.get_one::<String>(repo_branch_key) {
                let obj = match snap_repo_object
                    .revparse_single(format!("origin/{}", snap_repo_branch).as_str())
                {
                    Ok(obj) => obj,
                    Err(e) => {
                        eprintln!("Error finding branch: {}", e);
                        std::process::exit(1);
                    }
                };
                match snap_repo_object.checkout_tree(&obj, None) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error checking out branch: {}", e);
                        std::process::exit(1);
                    }
                }
                match snap_repo_object
                    .set_head(format!("refs/remotes/origin/{}", snap_repo_branch).as_str())
                {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error setting HEAD: {}", e);
                        std::process::exit(1);
                    }
                }
                let head = snap_repo_object.head().unwrap();
                let oid = head.target().unwrap();
                let commit = snap_repo_object.find_commit(oid).unwrap();
                match snap_repo_object.branch(snap_repo_branch, &commit, false) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error creating branch: {}", e);
                        std::process::exit(1);
                    }
                }
                match snap_repo_object.set_head(format!("refs/heads/{}", snap_repo_branch).as_str())
                {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error setting HEAD: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
