use serde_json::{json, Value};

pub struct OutputField {
    name: &'static str,
    description: &'static str,
    value: Value,
}

impl OutputField {
    pub fn new(name: &'static str, description: &'static str, value: Value) -> Self {
        Self {
            name,
            description,
            value,
        }
    }
}

pub fn explained(command: &str, description: &str, fields: Vec<OutputField>) -> String {
    explained_with_raw(command, description, fields, Value::Null)
}

pub fn error(command: &str, description: &str, message: &str) -> String {
    explained(
        command,
        description,
        vec![
            field("status", "명령 실행 결과", json!("error")),
            field("message", "오류 내용", json!(message)),
        ],
    )
}

pub fn explained_with_raw(
    command: &str,
    description: &str,
    fields: Vec<OutputField>,
    raw: Value,
) -> String {
    let fields = fields
        .into_iter()
        .map(|field| {
            json!({
                "name": field.name,
                "description": field.description,
                "value": field.value,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "command": command,
        "description": description,
        "fields": fields,
        "raw": raw,
    })
    .to_string()
}

pub fn field(name: &'static str, description: &'static str, value: Value) -> OutputField {
    OutputField::new(name, description, value)
}

pub fn parse_json_or_text(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!(value))
}
