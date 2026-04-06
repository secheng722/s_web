//! Enhanced Swagger generation with custom configuration support

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Swagger configuration for a route
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwaggerInfo {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub responses: HashMap<String, ApiResponse>,
    pub request_body: Option<RequestBody>,
    pub security: Vec<SecurityRequirement>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_: String,
    pub description: Option<String>,
    pub required: bool,
    pub schema: Schema,
}

/// API response definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
}

/// Request body definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: bool,
}

/// Media type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
    pub example: Option<Value>,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub type_: String,
    pub format: Option<String>,
    pub example: Option<Value>,
    pub properties: Option<HashMap<String, Schema>>,
    pub items: Option<Box<Schema>>,
}

impl Schema {
    pub fn string() -> Self {
        Self {
            type_: "string".to_string(),
            format: None,
            example: None,
            properties: None,
            items: None,
        }
    }

    pub fn object() -> Self {
        Self {
            type_: "object".to_string(),
            format: None,
            example: None,
            properties: None,
            items: None,
        }
    }
}

/// Security requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    pub name: String,
    pub scopes: Vec<String>,
}

/// Builder pattern for creating SwaggerInfo
pub struct SwaggerBuilder {
    info: SwaggerInfo,
}

impl SwaggerBuilder {
    pub fn new() -> Self {
        Self {
            info: SwaggerInfo::default(),
        }
    }

    pub fn summary<S: Into<String>>(mut self, summary: S) -> Self {
        self.info.summary = Some(summary.into());
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.info.description = Some(description.into());
        self
    }

    pub fn tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.info.tags.push(tag.into());
        self
    }

    pub fn parameter<S: Into<String>, T: Into<String>>(
        mut self,
        name: S,
        in_: T,
        description: Option<String>,
        required: bool,
    ) -> Self {
        self.info.parameters.push(Parameter {
            name: name.into(),
            in_: in_.into(),
            description,
            required,
            schema: Schema::string(),
        });
        self
    }

    pub fn path_param<S: Into<String>>(self, name: S, description: S) -> Self {
        self.parameter(name, "path", Some(description.into()), true)
    }

    pub fn query_param<S: Into<String>>(self, name: S, description: S, required: bool) -> Self {
        self.parameter(name, "query", Some(description.into()), required)
    }

    pub fn response<S: Into<String>>(mut self, status: S, description: S) -> Self {
        self.info.responses.insert(
            status.into(),
            ApiResponse {
                description: description.into(),
                content: None,
            },
        );
        self
    }

    pub fn json_response<S: Into<String>>(
        mut self,
        status: S,
        description: S,
        example: Option<Value>,
    ) -> Self {
        let mut content = HashMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Schema::object(),
                example: example.clone(),
            },
        );

        self.info.responses.insert(
            status.into(),
            ApiResponse {
                description: description.into(),
                content: Some(content),
            },
        );
        self
    }

    pub fn request_body(mut self, example: Value) -> Self {
        let mut content = HashMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Schema::object(),
                example: Some(example.clone()),
            },
        );

        self.info.request_body = Some(RequestBody {
            description: Some("Request body".to_string()),
            content,
            required: true,
        });
        self
    }

    pub fn security<S: Into<String>>(mut self, name: S, scopes: Vec<String>) -> Self {
        self.info.security.push(SecurityRequirement {
            name: name.into(),
            scopes,
        });
        self
    }

    pub fn bearer_auth(mut self) -> Self {
        self.info.security.push(SecurityRequirement {
            name: "bearerAuth".to_string(),
            scopes: vec![],
        });
        self.response("401", "Unauthorized - Bearer token required")
    }

    pub fn success_responses(self) -> Self {
        self.response("200", "Success")
            .response("500", "Internal Server Error")
    }

    pub fn crud_responses(self) -> Self {
        self.response("200", "Success")
            .response("400", "Bad Request")
            .response("401", "Unauthorized")
            .response("404", "Not Found")
            .response("500", "Internal Server Error")
    }

    pub fn build(self) -> SwaggerInfo {
        self.info
    }
}

impl Default for SwaggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn swagger() -> SwaggerBuilder {
    SwaggerBuilder::new()
}

/// Generate enhanced OpenAPI JSON with custom swagger info
pub fn generate_enhanced_swagger_json(
    routes: &[(String, String)],
    custom_info: &HashMap<String, SwaggerInfo>,
) -> String {
    let mut paths = serde_json::Map::new();

    for (method, path) in routes {
        let openapi_path = convert_path_format(path);
        let route_key = format!("{}-{}", method.to_uppercase(), path);

        let path_item = paths
            .entry(openapi_path.clone())
            .or_insert_with(|| json!({}));

        if let Some(path_obj) = path_item.as_object_mut() {
            let operation = if let Some(custom) = custom_info.get(&route_key) {
                create_operation_from_custom(custom, path)
            } else {
                create_default_operation(method, path)
            };

            path_obj.insert(method.to_lowercase(), operation);
        }
    }

    let swagger_doc = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "s_web API",
            "version": "1.0.0",
            "description": "API documentation generated by s_web framework"
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "paths": paths
    });

    serde_json::to_string_pretty(&swagger_doc).unwrap_or_else(|e| {
        eprintln!("[s_web] swagger serialization error: {e}");
        String::from("{}")
    })
}

fn convert_path_format(path: &str) -> String {
    path.split('/')
        .map(|part| {
            if let Some(param_name) = part.strip_prefix(':') {
                format!("{{{}}}", param_name)
            } else if let Some(param_name) = part.strip_prefix('*') {
                format!("{{{}}}", param_name)
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Extract path parameters from a route pattern
fn extract_path_params(path: &str) -> Vec<(&str, bool)> {
    path.split('/')
        .filter_map(|part| {
            if let Some(name) = part.strip_prefix(':') {
                Some((name, false))
            } else if let Some(name) = part.strip_prefix('*') {
                Some((name, true))
            } else {
                None
            }
        })
        .collect()
}

fn string_param_json(name: &str, is_wildcard: bool) -> Value {
    let desc = if is_wildcard {
        format!("The {} wildcard parameter", name)
    } else {
        format!("The {} parameter", name)
    };
    json!({
        "name": name,
        "in": "path",
        "required": true,
        "schema": { "type": "string" },
        "description": desc
    })
}

fn create_operation_from_custom(custom: &SwaggerInfo, path: &str) -> Value {
    let mut operation = json!({
        "summary": custom.summary,
        "description": custom.description,
        "tags": custom.tags,
    });

    let mut parameters = custom.parameters.clone();
    for (name, is_wildcard) in extract_path_params(path) {
        if !parameters.iter().any(|p| p.name == name) {
            parameters.push(Parameter {
                name: name.to_string(),
                in_: "path".to_string(),
                description: Some(if is_wildcard {
                    format!("The {} wildcard parameter", name)
                } else {
                    format!("The {} parameter", name)
                }),
                required: true,
                schema: Schema::string(),
            });
        }
    }

    if !parameters.is_empty() {
        operation["parameters"] = serde_json::to_value(parameters).unwrap_or(json!([]));
    }

    if !custom.responses.is_empty() {
        operation["responses"] = serde_json::to_value(&custom.responses).unwrap_or(json!({}));
    } else {
        operation["responses"] = json!({ "200": { "description": "Success" } });
    }

    if let Some(request_body) = &custom.request_body {
        operation["requestBody"] = serde_json::to_value(request_body).unwrap_or(json!({}));
    }

    if !custom.security.is_empty() {
        let security_array: Vec<Value> = custom
            .security
            .iter()
            .map(|req| json!({ req.name.clone(): req.scopes }))
            .collect();
        operation["security"] = json!(security_array);
    }

    operation
}

fn create_default_operation(method: &str, path: &str) -> Value {
    let mut operation = json!({
        "summary": format!("{} {}", method, path),
        "responses": {
            "200": { "description": "Success" }
        }
    });

    let parameters: Vec<Value> = extract_path_params(path)
        .iter()
        .map(|(name, is_wildcard)| string_param_json(name, *is_wildcard))
        .collect();

    if !parameters.is_empty() {
        operation["parameters"] = json!(parameters);
    }

    if matches!(method, "POST" | "PUT" | "PATCH") {
        operation["requestBody"] = json!({
            "required": true,
            "content": {
                "application/json": {
                    "schema": { "type": "object" }
                }
            }
        });
    }

    operation
}

pub fn generate_swagger_ui(json_url: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>s_web API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
    <style>
        body {{ margin: 0; padding: 0; }}
        .swagger-ui .topbar {{ display: none; }}
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({{
            url: '{json_url}',
            dom_id: '#swagger-ui',
            presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.presets.standalone],
            tryItOutEnabled: true,
            showRequestHeaders: true,
            docExpansion: 'list',
            filter: true,
            showExtensions: true,
            showCommonExtensions: true
        }});
    </script>
</body>
</html>
    "#,
        json_url = json_url
    )
}
