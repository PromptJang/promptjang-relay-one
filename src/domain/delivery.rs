use crate::domain::DomainError;
use standardwebhooks::Webhook;

#[cfg(test)]
pub const RETRY_DELAYS: [i64; 5] = [60, 120, 240, 480, 960];
#[cfg(test)]
const MAX_RESPONSE_BYTES: usize = 10_240;

#[cfg(test)]
pub fn retry_delay(retry_count: i32) -> Option<i64> {
    RETRY_DELAYS.get(retry_count.max(0) as usize).copied()
}

#[cfg(test)]
pub fn truncate_body(body: String) -> String {
    if body.len() <= MAX_RESPONSE_BYTES {
        return body;
    }
    let mut boundary = MAX_RESPONSE_BYTES;
    while !body.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}[truncated]", &body[..boundary])
}

pub fn signature(
    secret: &str,
    event_id: &str,
    timestamp: i64,
    payload: &[u8],
) -> Result<String, DomainError> {
    Webhook::new(secret)
        .map_err(|error| DomainError::internal(format!("signing key rejected: {error}")))?
        .sign(event_id, timestamp, payload)
        .map_err(|error| DomainError::internal(format!("payload signing failed: {error}")))
}

pub fn signature_header(
    current_secret: &str,
    previous_secret: Option<&str>,
    event_id: &str,
    timestamp: i64,
    payload: &[u8],
) -> Result<String, DomainError> {
    let mut signatures = vec![signature(current_secret, event_id, timestamp, payload)?];
    if let Some(previous_secret) = previous_secret {
        signatures.push(signature(previous_secret, event_id, timestamp, payload)?);
    }
    Ok(signatures.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_retry_schedule_matches_cloud() {
        // Arrange
        let expected = vec![60, 120, 240, 480, 960];

        // Act
        let delays = (0..5).filter_map(retry_delay).collect::<Vec<_>>();

        // Assert
        assert_eq!(delays, expected);
    }

    #[test]
    fn retry_delay_is_none_past_the_last_attempt() {
        // Arrange
        let retry_count = 5;

        // Act
        let delay = retry_delay(retry_count);

        // Assert
        assert_eq!(delay, None);
    }

    #[test]
    fn negative_retry_counts_read_the_first_slot() {
        // Arrange
        let retry_count = -1;

        // Act
        let delay = retry_delay(retry_count);

        // Assert
        assert_eq!(delay, Some(60));
    }

    #[test]
    fn short_bodies_pass_through_unchanged() {
        // Arrange
        let body = "{\"ok\":true}".to_string();

        // Act
        let truncated = truncate_body(body.clone());

        // Assert
        assert_eq!(truncated, body);
    }

    #[test]
    fn long_bodies_are_marked_as_truncated() {
        // Arrange
        let body = "x".repeat(MAX_RESPONSE_BYTES + 100);

        // Act
        let truncated = truncate_body(body);

        // Assert
        assert!(truncated.ends_with("[truncated]"));
        assert!(truncated.len() <= MAX_RESPONSE_BYTES + "[truncated]".len());
    }

    #[test]
    fn truncation_respects_multibyte_boundaries() {
        // Arrange
        let body = "あ".repeat(MAX_RESPONSE_BYTES);

        // Act
        let truncated = truncate_body(body);

        // Assert
        assert!(truncated.ends_with("[truncated]"));
        assert!(truncated.chars().all(|character| character == 'あ'
            || character == '['
            || character == 't'
            || character == 'r'
            || character == 'u'
            || character == 'n'
            || character == 'c'
            || character == 'a'
            || character == 'e'
            || character == 'd'
            || character == ']'));
    }

    #[test]
    fn signing_fixture_is_stable() {
        // Arrange
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/standard-webhooks-v1.json"
        ))
        .expect("fixture is valid JSON");
        let secret = fixture["secret"].as_str().expect("secret");
        let event_id = fixture["event_id"].as_str().expect("event ID");
        let timestamp = fixture["timestamp"].as_i64().expect("timestamp");
        let payload = fixture["payload"].as_str().expect("payload").as_bytes();
        let expected = fixture["signature"].as_str().expect("signature");

        // Act
        let signed = signature(secret, event_id, timestamp, payload);

        // Assert
        assert_eq!(signed.as_deref(), Ok(expected));
    }

    #[test]
    fn signature_changes_with_event_payload_or_secret() {
        // Arrange
        let secret = "whsec_C2FVsBQIhrscChlQIMV+b5sSYspob7oD";
        let other_secret = "whsec_dGVzdF9zZWNyZXRfZm9yX3NpZ25pbmc=";
        let base = signature(secret, "event-1", 1_000, b"{}").expect("signing succeeds");
        let other_event = signature(secret, "event-2", 1_000, b"{}").expect("signing succeeds");
        let other_payload = signature(secret, "event-1", 1_000, b"{ }").expect("signing succeeds");
        let other_secret =
            signature(other_secret, "event-1", 1_000, b"{}").expect("signing succeeds");

        // Act
        let all_distinct = base != other_event && base != other_payload && base != other_secret;

        // Assert
        assert!(all_distinct);
    }

    #[test]
    fn v02_hex_secret_suffix_is_accepted_as_base64() {
        // Arrange
        let secret = format!("whsec_{}", "0123456789abcdef".repeat(4));

        // Act
        let signed = signature(&secret, "event-1", 1_000, b"{}");

        // Assert
        assert!(signed.is_ok());
    }

    #[test]
    fn rotation_serializes_two_versioned_signatures_in_one_header() {
        // Arrange
        let current = "whsec_C2FVsBQIhrscChlQIMV+b5sSYspob7oD";
        let previous = "whsec_dGVzdF9zZWNyZXRfZm9yX3NpZ25pbmc=";

        // Act
        let header = signature_header(current, Some(previous), "event-1", 1_000, b"{}")
            .expect("signatures are valid");
        let signatures = header.split(' ').collect::<Vec<_>>();

        // Assert
        assert_eq!(signatures.len(), 2);
        assert!(signatures.iter().all(|value| value.starts_with("v1,")));
    }
}
