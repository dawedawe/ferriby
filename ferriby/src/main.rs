use crate::app::{App, Source};
use config::{Config, File};
use git::GitSource;
use github::GitHubSource;
use std::env;

pub mod app;
pub mod event;
pub mod git;
pub mod github;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();

    let sources = parse_args(&args);
    match sources {
        Ok(sources) => {
            let terminal = ratatui::init();
            let result = App::new(sources).run(terminal).await;
            ratatui::restore();
            result
        }
        Err(e) => {
            eprintln!("{e}");
            usage();
        }
    }
}

const GH_PAT_ENV_NAME: &str = "FERRIBY_GH_PAT";

fn config_path() -> String {
    env::home_dir()
        .map(|mut h| {
            if std::env::consts::OS == "windows" {
                h.push("AppData");
                h.push("Roaming");
                h.push("ferriby");
                h.push("config.json");
            } else {
                h.push(".config");
                h.push("ferriby");
                h.push("config.json");
            };
            h.to_str()
                .expect("failed to convert PathBuf to &str")
                .to_string()
        })
        .expect("failed to determine config path")
}

fn configured_sources(path: &str) -> Result<Vec<Source>, String> {
    let settings = Config::builder()
        .add_source(File::with_name(path))
        .build()
        .map_err(|_| format!("failed to parse config file {path}").to_string())?;
    let mut sources = vec![];

    let git_config = settings.get_array("git");
    if let Ok(paths) = git_config {
        paths.iter().for_each(|path| {
            let source = Source::Git(GitSource {
                path: path.to_string(),
            });
            sources.push(source);
        })
    };

    let gh_config = settings.get_array("github");
    if let Ok(repos) = gh_config {
        let pat = match std::env::var(GH_PAT_ENV_NAME) {
            Ok(token) if !token.is_empty() => Some(token),
            _ => None,
        };
        repos.iter().for_each(|repo| {
            let repo = repo.to_string();
            let gh_args: Vec<&str> = repo.split("/").collect();
            if gh_args.len() < 2 {
                panic!("invalid GitHub argument format, expected 'owner/repo'.");
            }

            let s = Source::GitHub(GitHubSource {
                owner: gh_args[0].to_string(),
                repo: gh_args[1].to_string(),
                pat: pat.clone(),
            });
            sources.push(s);
        })
    };

    if sources.is_empty() {
        Err("no sources defined in config file".into())
    } else {
        Ok(sources)
    }
}

fn parse_args(args: &[String]) -> Result<Vec<Source>, String> {
    if args.len() <= 1 {
        let path = config_path();
        configured_sources(path.as_str())
    } else if args.len() == 3 && args[1] == "-c" {
        let path = args[2].as_str();
        configured_sources(path)
    } else {
        let chunks = args[1..].chunks(2);
        let mut sources = vec![];
        for chunk in chunks {
            if chunk.len() != 2 {
                return Err("argument missing".into());
            }

            if chunk[0] == "-gh" {
                let pat = match std::env::var(GH_PAT_ENV_NAME) {
                    Ok(token) if !token.is_empty() => Some(token),
                    _ => None,
                };
                let gh_arg: Vec<&str> = chunk[1].split("/").collect();
                if gh_arg.len() < 2 {
                    return Err("Invalid GitHub argument format. Expected 'owner/repo'.".into());
                }
                let github_source = GitHubSource {
                    owner: gh_arg[0].to_string(),
                    repo: gh_arg[1].to_string(),
                    pat,
                };
                sources.push(Source::GitHub(github_source));
            } else if chunk[0] == "-g" {
                let git_source = GitSource {
                    path: chunk[1].clone(),
                };
                sources.push(Source::Git(git_source));
            } else if chunk[0] == "-c" {
                return Err("-c arg can't be combined with other args".into());
            } else {
                return Err("unknown argument".into());
            };
        }

        Ok(sources)
    }
}

fn usage() -> ! {
    eprintln!("Usage: ferriby [-c config_file] | [-gh owner/repository] [-g path_to_repo]");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_args_returns_err_for_mutual_exclusive_args() {
        let args = vec![
            "ferriby".into(),
            "-gh".into(),
            "owner1/repo1".into(),
            "-f".into(),
            "foo/config.json".into(),
        ];
        let sources = parse_args(&args);

        assert!(sources.is_err());
    }

    #[test]
    fn parse_args_returns_err_for_missing_arg() {
        let args = vec!["ferriby".into(), "-gh".into()];
        let sources = parse_args(&args);
        assert!(sources.is_err());
    }

    #[test]
    fn parse_args_returns_err_for_unknown_arg() {
        let args = vec!["ferriby".into(), "-xxx".into()];
        let sources = parse_args(&args);
        assert!(sources.is_err());
    }

    #[test]
    fn parse_args_returns_sources() {
        let args = vec![
            "ferriby".into(),
            "-gh".into(),
            "owner1/repo1".into(),
            "-g".into(),
            "dir1/repo2".into(),
            "-gh".into(),
            "owner2/repo3".into(),
        ];
        let sources = parse_args(&args);

        assert!(sources.is_ok());
        let sources = sources.unwrap();
        assert_eq!(sources.len(), 3);

        if let Source::GitHub(GitHubSource {
            owner,
            repo,
            pat: _,
        }) = &sources[0]
        {
            assert_eq!(owner, "owner1");
            assert_eq!(repo, "repo1");
        }

        if let Source::Git(GitSource { path }) = &sources[1] {
            assert_eq!(path, "dir1/repo2");
        }

        if let Source::GitHub(GitHubSource {
            owner,
            repo,
            pat: _,
        }) = &sources[2]
        {
            assert_eq!(owner, "owner2");
            assert_eq!(repo, "repo3");
        }
    }

    #[test]
    fn empty_config_file_should_err() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let sources = configured_sources(path);
        assert!(sources.is_err());
    }

    #[test]
    fn config_file_with_empty_json_should_err() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{}}").unwrap();
        temp_file.flush().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let sources = configured_sources(path);
        assert!(sources.is_err());
    }

    #[test]
    fn config_file_with_just_empty_arrays_should_err() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{ \"git\": [], \"github\": [] }}").unwrap();
        temp_file.flush().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let sources = configured_sources(path);
        assert!(sources.is_err());
    }

    #[test]
    fn config_file_sources_are_parsed_correctly() {
        let mut temp_file = tempfile::Builder::new()
            .suffix(".json")
            .tempfile()
            .expect("NamedTempFile::new() failed");
        let config = "{ \
                \"git\": [ \
                    \"foo/bar/baz\", \
                    \"mi/mu/meh\" \
                ], \
                \"github\": [ \
                    \"owner/repo1\", \
                    \"owner/repo2\", \
                    \"owner/repo3\" \
                ] \
            }";
        temp_file
            .write_all(config.as_bytes())
            .expect("write_all failed");
        temp_file.flush().expect("flush failed");

        let path = temp_file.path().to_str().unwrap();
        let sources = configured_sources(path);
        match sources {
            Ok(sources) => {
                assert_eq!(sources.len(), 5);
            }
            Err(_) => assert!(sources.is_ok()),
        }
    }
}
