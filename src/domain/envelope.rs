use super::DomainError;
use serde_json::Value;
use std::sync::OnceLock;

static VALIDATOR: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

/// Only explicitly versioned agent messages opt into validation.
pub fn validate(value: &Value) -> Result<(), DomainError> {
    let Some(name) = value.get("schema").and_then(Value::as_str) else {
        return Ok(());
    };
    if !name.starts_with("promptjang.agent-message.") {
        return Ok(());
    }
    if name != "promptjang.agent-message.v1" {
        return Err(DomainError::bad_request(
            "unsupported agent message envelope version",
        ));
    }
    let validator = VALIDATOR
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(include_str!(
                "../../skills/promptjang/references/agent-message-v1.schema.json"
            ))
            .map_err(|error| error.to_string())?;
            jsonschema::options()
                .should_validate_formats(true)
                .build(&schema)
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|_| DomainError::internal("agent envelope schema unavailable"))?;
    if !validator.is_valid(value) {
        return Err(DomainError::bad_request(
            "invalid Agent Message Envelope v1; see the task/result JSON Schema",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_payloads_remain_valid() {
        for value in [
            json!("text"),
            json!({"kind":"task","task":"legacy"}),
            json!([1, 2]),
        ] {
            assert!(validate(&value).is_ok());
        }
    }

    #[test]
    fn task_and_result_contracts_are_validated() {
        let task = json!({"schema":"promptjang.agent-message.v1","kind":"task","correlation_id":"one","task":"Review"});
        assert!(validate(&task).is_ok());
        let mut result = json!({"schema":"promptjang.agent-message.v1","kind":"result","correlation_id":"one","in_reply_to":"550e8400-e29b-41d4-a716-446655440000","status":"succeeded","summary":"Done"});
        assert!(validate(&result).is_ok());
        result["status"] = json!("failed");
        assert!(validate(&result).is_err());
        result["error"] = json!("Unable to complete");
        assert!(validate(&result).is_ok());
        result["in_reply_to"] = json!("not-a-uuid");
        assert!(validate(&result).is_err());
        assert!(validate(&json!({"schema":"promptjang.agent-message.v2"})).is_err());
        assert!(
            validate(&json!({"schema":"promptjang.agent-message.v1","kind":"task","task":""}))
                .is_err()
        );
    }
}
