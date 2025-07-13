use http::{HeaderMap, Method};
use reqwest::Url;

pub async fn get_with_headers(url: Url, header_map: HeaderMap) -> Option<String> {
    let mut request = reqwest::Request::new(Method::GET, url);

    header_map.into_iter().for_each(|header| {
        request
            .headers_mut()
            .insert(header.0.expect("expected HeaderName"), header.1);
    });

    let client = reqwest::Client::new();
    match client
        .execute(request)
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(response) => {
            let bytes = response.bytes().await.expect("bytes() failed");
            let body_str = std::str::from_utf8(&bytes).expect("from_utf8() failed");
            Some(body_str.to_string())
        }
        Err(_) => None,
    }
}
