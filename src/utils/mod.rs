use std::collections::HashMap;
use url::form_urlencoded;

pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

pub fn url_decode(input: &str) -> String {
    form_urlencoded::parse(input.as_bytes())
        .map(|(key, val)| format!("{key}={val}"))
        .collect::<Vec<_>>()
        .join("&")
}

pub fn parse_content_type(content_type: &str) -> (String, HashMap<String, String>) {
    let mut parts = content_type.split(';');
    let media_type = parts.next().unwrap_or("").trim().to_lowercase();

    let mut parameters = HashMap::new();
    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            parameters.insert(
                key.trim().to_lowercase(),
                value.trim().trim_matches('"').to_string(),
            );
        }
    }

    (media_type, parameters)
}
