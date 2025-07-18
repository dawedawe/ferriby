use crate::app::{App, Source};
use codeberg::CodebergSource;
use config::{Config, File, Value};
use git::GitSource;
use github::GitHubSource;
use gitlab::GitLabSource;
use std::env;

pub mod app;
pub mod codeberg;
pub mod event;
pub mod git;
pub mod githoster;
pub mod github;
pub mod gitlab;
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

const CB_PAT_ENV_NAME: &str = "FERRIBY_CB_PAT";
const GH_PAT_ENV_NAME: &str = "FERRIBY_GH_PAT";
const GL_PAT_ENV_NAME: &str = "FERRIBY_GL_PAT";

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
        .map_err(|_| format!("failed to parse config file {path}"))?;
    let mut sources = vec![];

    let git_config = settings.get_array("git");
    if let Ok(paths) = git_config {
        paths.iter().for_each(|path| {
            let source = Source::Git(GitSource {
                path: path.clone().into_string().expect("expected a string"),
            });
            sources.push(source);
        })
    };

    handle_git_hoster_config(
        &settings,
        &mut sources,
        "github",
        GH_PAT_ENV_NAME,
        |owner, repo, pat| {
            Source::GitHub(GitHubSource {
                owner,
                repo,
                pat: pat.clone(),
            })
        },
    );

    handle_git_hoster_config(
        &settings,
        &mut sources,
        "codeberg",
        CB_PAT_ENV_NAME,
        |owner, repo, pat| {
            Source::Codeberg(CodebergSource {
                owner,
                repo,
                pat: pat.clone(),
            })
        },
    );

    let gitlab_config = settings.get_array("gitlab");
    if let Ok(tables) = gitlab_config {
        let pat = try_get_pat(GL_PAT_ENV_NAME);

        tables.iter().for_each(|table| {
            let pat = pat.clone();
            let table = table.clone().into_table().expect("expected a table");
            let hostname_value = table
                .get("hostname")
                .expect("expected a hostname key")
                .clone();
            let hostname = hostname_value.into_string().expect("expected a string");
            let project_id_value = table
                .get("projectid")
                .expect("expected a projectid key")
                .clone();
            let project_id = project_id_value.into_string().expect("expected a string");
            let project_name_value = table
                .get("projectname")
                .expect("expected a projectname key")
                .clone();
            let project_name = project_name_value.into_string().expect("expected a string");

            let pat = if pat.is_none() {
                table
                    .get("pat")
                    .map(|v| v.clone().into_string().expect("expected a string"))
            } else {
                pat
            };

            let source = Source::GitLab(GitLabSource {
                hostname,
                project_id,
                project_name,
                pat,
            });
            sources.push(source);
        })
    };

    if sources.is_empty() {
        Err("no sources defined in config file".into())
    } else {
        Ok(sources)
    }
}

fn handle_git_hoster_config<F>(
    settings: &Config,
    sources: &mut Vec<Source>,
    key: &str,
    pat_env_var: &str,
    f: F,
) where
    F: Fn(String, String, Option<String>) -> Source,
{
    let cb_config = settings.get_array(key);
    if let Ok(repos) = cb_config {
        let pat = try_get_pat(pat_env_var);
        repos.iter().for_each(|conf_val| {
            let (owner, repo) = parse_owner_repo_conf_value(conf_val);
            let s = f(owner, repo, pat.clone());
            sources.push(s);
        })
    };
}

fn try_get_pat(env_var: &str) -> Option<String> {
    match std::env::var(env_var) {
        Ok(token) if !token.is_empty() => Some(token),
        _ => None,
    }
}

fn parse_owner_repo(val: &str) -> (String, String) {
    let parts: Vec<&str> = val.split("/").collect();
    if parts.len() != 2 {
        panic!("invalid argument format, expected 'owner/repo'.");
    }
    (parts[0].to_string(), parts[1].to_string())
}

fn parse_owner_repo_conf_value(conf_val: &Value) -> (String, String) {
    let val = conf_val.clone().into_string().expect("expected a string");
    parse_owner_repo(&val)
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
                let (owner, repo) = parse_owner_repo(&chunk[1]);
                let source = GitHubSource { owner, repo, pat };
                sources.push(Source::GitHub(source));
            } else if chunk[0] == "-gl" {
                let pat = match std::env::var(GL_PAT_ENV_NAME) {
                    Ok(token) if !token.is_empty() => Some(token),
                    _ => None,
                };
                let parts: Vec<&str> = chunk[1].splitn(3, "/").collect();
                if parts.len() != 3 {
                    panic!("invalid argument format, expected 'hostname/projectid/projectname'.");
                }
                let source = GitLabSource {
                    hostname: parts[0].to_string(),
                    project_id: parts[1].to_string(),
                    project_name: parts[2].to_string(),
                    pat,
                };
                sources.push(Source::GitLab(source));
            } else if chunk[0] == "-cb" {
                let pat = match std::env::var(CB_PAT_ENV_NAME) {
                    Ok(token) if !token.is_empty() => Some(token),
                    _ => None,
                };
                let (owner, repo) = parse_owner_repo(&chunk[1]);
                let source = CodebergSource { owner, repo, pat };
                sources.push(Source::Codeberg(source));
            } else if chunk[0] == "-g" {
                let source = GitSource {
                    path: chunk[1].clone(),
                };
                sources.push(Source::Git(source));
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
    eprintln!(
        "Usage: ferriby [-c config_file] | [-g path_to_repo] [-gh owner/repository] [-cb owner/repository] [-gl hostname/projectid/projectname]"
    );
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
            "-cb".into(),
            "owner2/repo3".into(),
            "-gl".into(),
            "gitlab.com/12345/proj1".into(),
        ];
        let sources = parse_args(&args);

        assert!(sources.is_ok());
        let sources = sources.unwrap();
        assert_eq!(sources.len(), 4);

        if let Source::GitHub(GitHubSource {
            owner,
            repo,
            pat: _,
        }) = &sources[0]
        {
            assert_eq!(owner, "owner1");
            assert_eq!(repo, "repo1");
        } else {
            panic!("unexpected source");
        }

        if let Source::Git(GitSource { path }) = &sources[1] {
            assert_eq!(path, "dir1/repo2");
        } else {
            panic!("unexpected source");
        }

        if let Source::Codeberg(CodebergSource {
            owner,
            repo,
            pat: _,
        }) = &sources[2]
        {
            assert_eq!(owner, "owner2");
            assert_eq!(repo, "repo3");
        } else {
            panic!("unexpected source");
        }

        if let Source::GitLab(GitLabSource {
            hostname,
            project_id,
            project_name,
            pat: _,
        }) = &sources[3]
        {
            assert_eq!(hostname, "gitlab.com");
            assert_eq!(project_id, "12345");
            assert_eq!(project_name, "proj1");
        } else {
            panic!("unexpected source");
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
        writeln!(
            temp_file,
            "{{ \"git\": [], \"github\": [], \"codeberg\": [] }}"
        )
        .unwrap();
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
                    \"gh_owner1/gh_repo1\", \
                    \"gh_owner2/gh_repo2\", \
                    \"gh_owner3/gh_repo3\" \
                ], \
                \"codeberg\": [ \
                    \"cb_owner1/cb_repo1\", \
                    \"cb_owner2/cb_repo2\" \
                ], \
                \"gitlab\": [ \
                    { \"hostname\": \"gitlab.example.org\", \"projectid\": \"42\", \"projectname\": \"proj1\", \"pat\": \"glpat-123\" } \
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
                assert_eq!(sources.len(), 8);
                let g1_find = sources
                    .iter()
                    .find(|source| matches!(source, Source::Git(g) if g.path == "foo/bar/baz"));
                assert!(g1_find.is_some());

                let gh2_find = sources.iter().find(
                    |source| matches!(source, Source::GitHub(gh) if gh.owner == "gh_owner2" && gh.repo == "gh_repo2"),
                );
                assert!(gh2_find.is_some());

                let cb1_find = sources.iter().find(
                    |source| matches!(source, Source::Codeberg(cb) if cb.owner == "cb_owner1" && cb.repo == "cb_repo1"),
                );
                assert!(cb1_find.is_some());
                let cb2_find = sources.iter().find(
                    |source| matches!(source, Source::Codeberg(cb) if cb.owner == "cb_owner2" && cb.repo == "cb_repo2"),
                );
                assert!(cb2_find.is_some());

                let gl1_find =
                    sources.iter().find(
                    |source| matches!(source, Source::GitLab(gl)
                    if gl.hostname == "gitlab.example.org" && gl.project_id  == "42" && gl.project_name  == "proj1" && gl.pat  == Some("glpat-123".into())));
                assert!(gl1_find.is_some());
            }
            Err(_) => assert!(sources.is_ok()),
        }
    }

    #[test]
    fn try_get_pat_works_with_filled_env_var() {
        let key = "FERRIBY_TEST_PAT_XYZ";
        let value = "xyz";
        unsafe {
            env::set_var(key, value);
        }
        let pat = try_get_pat(key);
        assert!(pat.is_some_and(|p| p == value));
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn try_get_pat_returns_none_for_empty_env_var() {
        let key = "FERRIBY_TEST_PAT_EMPTY";
        unsafe {
            env::set_var(key, "");
        }
        let pat = try_get_pat(key);
        assert!(pat.is_none());
        unsafe {
            env::remove_var(key);
        }
    }
}
