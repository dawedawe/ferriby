use std::cell::LazyCell;

use chrono::NaiveDateTime;
use chrono::{DateTime, offset::Utc};
use http::{self, Method, header};
use regex::Regex;
use reqwest::Url;

#[derive(Debug, Clone)]
pub struct GitHubSource {
    pub owner: String,
    pub repo: String,
    pub pat: Option<String>,
}

impl Default for GitHubSource {
    fn default() -> Self {
        Self {
            owner: "rust-lang".into(),
            repo: "rust".into(),
            pat: None,
        }
    }
}

pub async fn get_last_event(source: GitHubSource) -> Option<DateTime<Utc>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/activity",
        source.owner, source.repo
    );
    let url = Url::parse(url.as_str()).expect("Url creation failed");
    let mut request = reqwest::Request::new(Method::GET, url);

    let headers = request.headers_mut();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("ferriby"),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        header::HeaderValue::from_static("2022-11-28"),
    );
    if let Some(token) = source.pat {
        let token = token.clone();
        let gh_pat = header::HeaderValue::from_str(format!("Bearer {token}").as_str())
            .expect("bad github pat");
        headers.insert(header::AUTHORIZATION, gh_pat);
    }

    let client = reqwest::Client::new();
    let response = client.execute(request).await.expect("http request failed");
    let bytes = response.bytes().await.expect("bytes() failed");
    let body_str = std::str::from_utf8(&bytes).expect("from_utf8() failed");
    let timestamps = parse_timestamps(body_str.to_string());
    timestamps.into_iter().max()
}

fn parse_timestamps(response: String) -> Vec<DateTime<Utc>> {
    let re: LazyCell<Regex> = LazyCell::new(|| {
        Regex::new("\"timestamp\":\"(\\d\\d\\d\\d-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\dZ)\"").unwrap()
    });

    re.captures_iter(&response)
        .map(|m| {
            let s = m.get(1).unwrap().as_str();
            let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                .expect("unexpected timestamp format");
            let secs = dt.and_utc().timestamp();
            DateTime::from_timestamp(secs, 0).expect("from_timestamp failed")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};

    use super::*;

    #[test]
    fn github_parse() {
        let s = String::from(
            "\"timestamp\":\"2025-05-16T20:41:19Z\" bla foo\
            \"timestamp\":\"2025-10-18T03:01:09Z\"",
        );
        let parsed = parse_timestamps(s);

        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].year(), 2025);
        assert_eq!(parsed[0].month(), 5);
        assert_eq!(parsed[0].day(), 16);
        assert_eq!(parsed[0].hour(), 20);
        assert_eq!(parsed[0].minute(), 41);
        assert_eq!(parsed[0].second(), 19);

        assert_eq!(parsed[1].year(), 2025);
        assert_eq!(parsed[1].month(), 10);
        assert_eq!(parsed[1].day(), 18);
        assert_eq!(parsed[1].hour(), 3);
        assert_eq!(parsed[1].minute(), 1);
        assert_eq!(parsed[1].second(), 9);
    }
}
