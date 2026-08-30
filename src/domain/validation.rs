use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use url::Url;

use crate::config::Config;
use crate::domain::DomainError;

pub fn validate_name(value: &str) -> Result<(), DomainError> {
    if value.trim().is_empty() || value.len() > 100 {
        Err(DomainError::bad_request(
            "name must contain 1 to 100 characters",
        ))
    } else {
        Ok(())
    }
}

pub fn extract_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .map(String::from)
}

pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

pub fn idempotency_key(headers: &HeaderMap) -> Result<Option<String>, DomainError> {
    match headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
    {
        None => Ok(None),
        Some(value) if value.is_empty() || value.len() > 255 => Err(DomainError::bad_request(
            "Idempotency-Key must contain 1 to 255 characters",
        )),
        Some(value) => Ok(Some(value.to_string())),
    }
}

pub fn ensure_payload_size(size: usize, maximum: usize) -> Result<(), DomainError> {
    if size > maximum {
        Err(DomainError::payload_too_large(format!(
            "payload exceeds configured maximum of {maximum} bytes"
        )))
    } else {
        Ok(())
    }
}

pub async fn validate_destination_url(raw: &str, config: &Config) -> Result<(), DomainError> {
    resolve_destination(raw, config).await.map(|_| ())
}

pub async fn resolve_destination(
    raw: &str,
    config: &Config,
) -> Result<(Url, String, SocketAddr), DomainError> {
    let url = Url::parse(raw).map_err(|_| DomainError::bad_request("invalid endpoint URL"))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(DomainError::bad_request(
            "destination must be an HTTP(S) URL without embedded credentials",
        ));
    }
    let host = url
        .host_str()
        .ok_or_else(|| DomainError::bad_request("endpoint host is required"))?
        .to_string();
    let port = url
        .port_or_known_default()
        .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
    let addresses = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|_| DomainError::bad_request("endpoint host could not be resolved"))?;
    let addresses = addresses.map(|address| address.ip()).collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(DomainError::bad_request(
            "destination host has no addresses",
        ));
    }
    let mut selected = None;
    for address in addresses {
        let private = !is_public_ip(address);
        if private
            && !config
                .allow_private_cidrs
                .iter()
                .any(|cidr| cidr.contains(&address))
        {
            return Err(DomainError::bad_request(
                "private destination address is not allowlisted",
            ));
        }
        if url.scheme() == "http" && (!private || !config.allow_insecure_http) {
            return Err(DomainError::bad_request(
                "HTTP is allowed only for allowlisted private destinations when PJ_ALLOW_INSECURE_HTTP=true",
            ));
        }
        selected.get_or_insert(SocketAddr::new(address, port));
    }
    Ok((
        url,
        host,
        selected
            .ok_or_else(|| DomainError::bad_request("destination host has no allowed addresses"))?,
    ))
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            !(v.is_private()
                || v.is_loopback()
                || v.is_link_local()
                || v.is_broadcast()
                || v.is_documentation()
                || v.is_multicast()
                || v.is_unspecified())
        }
        IpAddr::V6(v) => {
            !(v.is_loopback()
                || v.is_unspecified()
                || v.is_unique_local()
                || v.is_unicast_link_local()
                || v.is_multicast())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::domain::ErrorKind;
    use axum::http::HeaderValue;

    fn test_config() -> Config {
        Config::from_reader(|key| match key {
            "DATABASE_URL" => Some("postgres://localhost/relay".into()),
            "PJ_ENCRYPTION_KEY" => Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into()),
            _ => None,
        })
        .expect("test configuration")
    }

    #[test]
    fn extract_header_keeps_short_non_empty_values() {
        // Arrange
        let mut headers = HeaderMap::new();
        headers.insert("X-Event-Type", HeaderValue::from_static("order.created"));

        // Act
        let value = extract_header(&headers, "X-Event-Type");

        // Assert
        assert_eq!(value.as_deref(), Some("order.created"));
    }

    #[test]
    fn extract_header_drops_empty_and_overlong_values() {
        // Arrange
        let mut headers = HeaderMap::new();
        headers.insert("X-Empty", HeaderValue::from_static(""));
        headers.insert(
            "X-Long",
            HeaderValue::from_str(&"x".repeat(129)).expect("valid header value"),
        );

        // Act
        let empty = extract_header(&headers, "X-Empty");
        let long = extract_header(&headers, "X-Long");

        // Assert
        assert_eq!(empty, None);
        assert_eq!(long, None);
    }

    #[test]
    fn bearer_token_strips_the_scheme() {
        // Arrange
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer pj_oss_abc"));

        // Act
        let token = bearer_token(&headers);

        // Assert
        assert_eq!(token, Some("pj_oss_abc"));
    }

    #[test]
    fn bearer_token_rejects_other_schemes() {
        // Arrange
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic abc"));

        // Act
        let token = bearer_token(&headers);

        // Assert
        assert_eq!(token, None);
    }

    #[test]
    fn idempotency_key_is_optional() {
        // Arrange
        let headers = HeaderMap::new();

        // Act
        let key = idempotency_key(&headers);

        // Assert
        assert_eq!(key, Ok(None));
    }

    #[test]
    fn idempotency_key_accepts_valid_range() {
        // Arrange
        let mut headers = HeaderMap::new();
        headers.insert("Idempotency-Key", HeaderValue::from_static("order-1042"));
        let mut edge = HeaderMap::new();
        edge.insert(
            "Idempotency-Key",
            HeaderValue::from_str(&"k".repeat(255)).expect("valid header value"),
        );

        // Act
        let typical = idempotency_key(&headers);
        let boundary = idempotency_key(&edge);

        // Assert
        assert_eq!(typical, Ok(Some("order-1042".into())));
        assert!(matches!(boundary, Ok(Some(value)) if value.len() == 255));
    }

    #[test]
    fn idempotency_key_rejects_empty_and_overlong() {
        // Arrange
        let mut empty = HeaderMap::new();
        empty.insert("Idempotency-Key", HeaderValue::from_static(""));
        let mut long = HeaderMap::new();
        long.insert(
            "Idempotency-Key",
            HeaderValue::from_str(&"k".repeat(256)).expect("valid header value"),
        );

        // Act
        let empty_result = idempotency_key(&empty);
        let long_result = idempotency_key(&long);

        // Assert
        assert_eq!(empty_result.unwrap_err().kind, ErrorKind::BadRequest);
        assert_eq!(long_result.unwrap_err().kind, ErrorKind::BadRequest);
    }

    #[test]
    fn payload_size_allows_up_to_one_mib() {
        // Arrange
        let sizes = [0, 1024 * 1024];

        for size in sizes {
            // Act
            let result = ensure_payload_size(size, 1024 * 1024);

            // Assert
            assert_eq!(result, Ok(()));
        }
    }

    #[test]
    fn payload_size_rejects_over_one_mib() {
        // Arrange
        let size = 1024 * 1024 + 1;

        // Act
        let result = ensure_payload_size(size, 1024 * 1024);

        // Assert
        assert_eq!(result.unwrap_err().kind, ErrorKind::PayloadTooLarge);
    }

    #[test]
    fn private_v4_addresses_are_rejected() {
        // Arrange
        let addresses = [
            "10.0.0.1".parse::<IpAddr>().expect("valid IPv4"),
            "192.168.1.1".parse::<IpAddr>().expect("valid IPv4"),
            "127.0.0.1".parse::<IpAddr>().expect("valid IPv4"),
            "169.254.1.1".parse::<IpAddr>().expect("valid IPv4"),
            "0.0.0.0".parse::<IpAddr>().expect("valid IPv4"),
        ];

        for address in addresses {
            // Act
            let public = is_public_ip(address);

            // Assert
            assert!(!public, "{address} must not be public");
        }
    }

    #[test]
    fn public_and_private_v6_addresses_are_distinguished() {
        // Arrange
        let public = "2606:4700::1111".parse::<IpAddr>().expect("valid IPv6");
        let loopback = "::1".parse::<IpAddr>().expect("valid IPv6");
        let unique_local = "fd00::1".parse::<IpAddr>().expect("valid IPv6");

        // Act
        let results = [
            is_public_ip(public),
            is_public_ip(loopback),
            is_public_ip(unique_local),
        ];

        // Assert
        assert!(results[0], "public IPv6 must be public");
        assert!(!results[1], "loopback must not be public");
        assert!(!results[2], "unique-local must not be public");
    }

    #[tokio::test]
    async fn https_url_with_plain_host_is_accepted() {
        // Arrange
        let raw = "https://example.com/hook";

        // Act
        let config = test_config();
        let result = validate_destination_url(raw, &config).await;

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn non_https_or_credentialed_urls_are_rejected() {
        // Arrange
        let cases = [
            "http://example.com/hook",
            "ftp://example.com/hook",
            "https://user:pass@example.com/hook",
            "not a url",
        ];

        for raw in cases {
            // Act
            let result = validate_destination_url(raw, &test_config()).await;

            // Assert
            assert_eq!(
                result.unwrap_err().kind,
                ErrorKind::BadRequest,
                "{raw} must be rejected"
            );
        }
    }

    #[tokio::test]
    async fn localhost_and_unresolvable_hosts_are_rejected() {
        // Arrange
        let cases = [
            "https://localhost/hook",
            "https://name.invalid.hook.example",
        ];

        for raw in cases {
            // Act
            let result = validate_destination_url(raw, &test_config()).await;

            // Assert
            assert_eq!(
                result.unwrap_err().kind,
                ErrorKind::BadRequest,
                "{raw} must be rejected"
            );
        }
    }

    #[test]
    fn accepts_typical_name() {
        // Arrange
        let name = "Order service";

        // Act
        let result = validate_name(name);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_whitespace_only_name() {
        // Arrange
        let name = "   ";

        // Act
        let result = validate_name(name);

        // Assert
        assert_eq!(result.unwrap_err().kind, ErrorKind::BadRequest);
    }

    #[test]
    fn rejects_overlong_name() {
        // Arrange
        let name = "a".repeat(101);

        // Act
        let result = validate_name(&name);

        // Assert
        assert_eq!(result.unwrap_err().kind, ErrorKind::BadRequest);
    }

    #[test]
    fn accepts_name_at_exactly_100_characters() {
        // Arrange
        let name = "a".repeat(100);

        // Act
        let result = validate_name(&name);

        // Assert
        assert_eq!(result, Ok(()));
    }
}
