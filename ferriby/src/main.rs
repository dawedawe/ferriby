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

    let sources = if args.len() <= 1 {
        configured_sources()
    } else {
        parse_args(&args)
    };
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

fn configured_sources() -> Result<Vec<Source>, String> {
    let path = std::env::home_dir()
        .map(|h| {
            format!(
                "{}/.config/ferriby/config.json",
                h.into_os_string()
                    .into_string()
                    .expect("failed to convert OsString to String")
            )
        })
        .expect("failed to determine config path");
    let settings = Config::builder()
        .add_source(File::with_name(path.as_str()))
        .build()
        .map_err(|_| "failed to parse config file".to_string())?;
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
                panic!("Invalid GitHub argument format. Expected 'owner/repo'.");
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
    if args.len() < 3 {
        return Err("arguments missing".into());
    }

    let chunks = args[1..].chunks(2);
    let mut sources = vec![];
    for chunk in chunks {
        if chunk.len() != 2 {
            return Err("argument missing".into());
        }

        let source = if chunk[0] == "-gh" {
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
            Source::GitHub(github_source)
        } else if chunk[0] == "-g" {
            let git_source = GitSource {
                path: chunk[1].clone(),
            };
            Source::Git(git_source)
        } else {
            return Err("unknown argument".into());
        };
        sources.push(source);
    }

    Ok(sources)
}

fn usage() -> ! {
    eprintln!("Usage: ferriby [-gh owner/repository] [-g path_to_repo]");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_returns_err_for_empty_args() {
        let args = vec!["ferriby".into()];
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
}
