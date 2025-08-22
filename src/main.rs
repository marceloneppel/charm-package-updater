use clap::{arg, command, value_parser};
use git2::Repository;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
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

    if let Some(snap_repo) = matches.get_one::<Url>("snaprepo") {
        if let Some(ppa) = matches.get_one::<String>("ppa") {
            let formatted_repo_name = snap_repo
                .as_str()
                .split('/')
                .last()
                .unwrap()
                .split('.')
                .next()
                .unwrap();
            let snap_repo_path = Path::join(Path::new(TEMP_DIR), Path::new(formatted_repo_name));
            let snap_file_path = snap_repo_path.join("snap/snapcraft.yaml");
            if !snap_file_path.exists() {
                eprintln!("Snap file not found at: {}", snap_file_path.display());
                std::process::exit(1);
            }
            let snap_file_content = match fs::read_to_string(snap_file_path.clone()) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading snap file: {}", e);
                    std::process::exit(1);
                }
            };
            let mut parsed_yaml: BTreeMap<String, Value> =
                match serde_yaml::from_str(snap_file_content.as_str()) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        eprintln!("Error parsing snap file: {}", e);
                        std::process::exit(1);
                    }
                };
            let package_repositories_map = match parsed_yaml.get_mut("package-repositories") {
                Some(Value::Sequence(sequence)) => sequence,
                _ => {
                    eprintln!("package-repositories not found or not a mapping in the snap file");
                    std::process::exit(1);
                }
            };
            package_repositories_map.push(Value::Mapping(Mapping::from_iter(vec![
                (
                    Value::String("type".to_string()),
                    Value::String("apt".to_string()),
                ),
                (
                    Value::String("ppa".to_string()),
                    Value::String(ppa.to_string()),
                ),
            ])));
            let new_snap_file_content = match serde_yaml::to_string(&parsed_yaml) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error converting YAML to string: {}", e);
                    std::process::exit(1);
                }
            };
            match fs::write(snap_file_path, new_snap_file_content) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error writing snap file: {}", e);
                    std::process::exit(1);
                }
            };
        }
    }
}
