use std::cell::LazyCell;

use chrono::{DateTime, offset::Utc};
use http::{self, Method, header};
use regex::Regex;
use reqwest::Url;

#[derive(Debug, Clone)]
pub struct CodebergSource {
    pub owner: String,
    pub repo: String,
    pub pat: Option<String>,
}

impl Default for CodebergSource {
    fn default() -> Self {
        Self {
            owner: "rust-lang".into(),
            repo: "rust".into(),
            pat: None,
        }
    }
}

pub async fn get_last_event(source: CodebergSource) -> Option<DateTime<Utc>> {
    let url = format!(
        "https://codeberg.org/api/v1/repos/{}/{}",
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
        header::HeaderValue::from_static("application/json"),
    );
    if let Some(token) = source.pat {
        let token = token.clone();
        let cb_pat = header::HeaderValue::from_str(format!("token {token}").as_str())
            .expect("bad codeberg pat");
        headers.insert(header::AUTHORIZATION, cb_pat);
    }

    let client = reqwest::Client::new();
    match client
        .execute(request)
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(response) => {
            let bytes = response.bytes().await.expect("bytes() failed");
            let body_str = std::str::from_utf8(&bytes).expect("from_utf8() failed");
            let timestamps = parse_timestamps(body_str.to_string());
            timestamps.into_iter().max()
        }
        Err(_) => None,
    }
}

fn parse_timestamps(response: String) -> Vec<DateTime<Utc>> {
    let re: LazyCell<Regex> = LazyCell::new(|| {
        Regex::new("\"updated_at\":\"(\\d\\d\\d\\d-\\d\\d-\\d\\dT\\d\\d:\\d\\d:\\d\\d[+,-]\\d\\d:\\d\\d)\"").unwrap()
    });

    re.captures_iter(&response)
        .map(|m| {
            let s = m.get(1).unwrap().as_str();
            let dt = chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z")
                .expect("unexpected timestamp format");
            let secs = dt.timestamp();
            DateTime::from_timestamp(secs, 0).expect("from_timestamp failed")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};

    use super::*;

    #[test]
    fn codeberg_parse_positive_offset() {
        let s = String::from(
            "\"updated_at\":\"2025-07-11T12:30:20+02:00\" bla foo\
            \"updated_at\":\"2025-07-11T13:31:22+02:00\"",
        );
        let parsed = parse_timestamps(s);

        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].year(), 2025);
        assert_eq!(parsed[0].month(), 7);
        assert_eq!(parsed[0].day(), 11);
        assert_eq!(parsed[0].hour(), 10);
        assert_eq!(parsed[0].minute(), 30);
        assert_eq!(parsed[0].second(), 20);

        assert_eq!(parsed[1].year(), 2025);
        assert_eq!(parsed[1].month(), 7);
        assert_eq!(parsed[1].day(), 11);
        assert_eq!(parsed[1].hour(), 11);
        assert_eq!(parsed[1].minute(), 31);
        assert_eq!(parsed[1].second(), 22);
    }

    #[test]
    fn codeberg_parse_negative_offset() {
        let s = String::from(
            "\"updated_at\":\"2025-07-11T12:30:20-02:00\" bla foo\
            \"updated_at\":\"2025-07-11T13:31:22-02:00\"",
        );
        let parsed = parse_timestamps(s);

        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].year(), 2025);
        assert_eq!(parsed[0].month(), 7);
        assert_eq!(parsed[0].day(), 11);
        assert_eq!(parsed[0].hour(), 14);
        assert_eq!(parsed[0].minute(), 30);
        assert_eq!(parsed[0].second(), 20);

        assert_eq!(parsed[1].year(), 2025);
        assert_eq!(parsed[1].month(), 7);
        assert_eq!(parsed[1].day(), 11);
        assert_eq!(parsed[1].hour(), 15);
        assert_eq!(parsed[1].minute(), 31);
        assert_eq!(parsed[1].second(), 22);
    }
}
