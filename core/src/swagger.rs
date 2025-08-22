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
    pub in_: String, // "path", "query", "header", "cookie"
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

/// Security requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    pub name: String,
    pub scopes: Vec<String>,
}

impl SwaggerInfo {
    /// Create a new SwaggerInfo instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Set summary
    pub fn summary<S: Into<String>>(mut self, summary: S) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Set description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a tag
    pub fn tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add a parameter
    pub fn parameter<S: Into<String>>(
        mut self,
        name: S,
        in_: S,
        description: Option<String>,
        required: bool,
    ) -> Self {
        self.parameters.push(Parameter {
            name: name.into(),
            in_: in_.into(),
            description,
            required,
            schema: Schema {
                type_: "string".to_string(),
                format: None,
                example: None,
                properties: None,
                items: None,
            },
        });
        self
    }

    /// Add a response
    pub fn response<S: Into<String>>(mut self, status: S, description: S) -> Self {
        let mut content = HashMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Schema {
                    type_: "object".to_string(),
                    format: None,
                    example: None,
                    properties: None,
                    items: None,
                },
                example: None,
            },
        );

        self.responses.insert(
            status.into(),
            ApiResponse {
                description: description.into(),
                content: Some(content),
            },
        );
        self
    }

    /// Add a JSON response with example
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
                schema: Schema {
                    type_: "object".to_string(),
                    format: None,
                    example: example.clone(),
                    properties: None,
                    items: None,
                },
                example,
            },
        );

        self.responses.insert(
            status.into(),
            ApiResponse {
                description: description.into(),
                content: Some(content),
            },
        );
        self
    }

    /// Set request body
    pub fn request_body(mut self, example: Value) -> Self {
        let mut content = HashMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Schema {
                    type_: "object".to_string(),
                    format: None,
                    example: Some(example.clone()),
                    properties: None,
                    items: None,
                },
                example: Some(example),
            },
        );

        self.request_body = Some(RequestBody {
            description: Some("Request body".to_string()),
            content,
            required: true,
        });
        self
    }

    /// Add security requirement
    pub fn security<S: Into<String>>(mut self, name: S, scopes: Vec<String>) -> Self {
        self.security.push(SecurityRequirement {
            name: name.into(),
            scopes,
        });
        self
    }

    /// Add bearer token authentication
    pub fn bearer_auth(mut self) -> Self {
        self.security.push(SecurityRequirement {
            name: "bearerAuth".to_string(),
            scopes: vec![],
        });
        self
    }
}

/// Generate enhanced OpenAPI JSON with custom swagger info
pub fn generate_enhanced_swagger_json(
    routes: &[(String, String)],
    custom_info: &HashMap<String, SwaggerInfo>,
) -> String {
    let mut paths = serde_json::Map::new();

    for (method, path) in routes {
        // Convert Ree path format (:id) to OpenAPI format ({id})
        let openapi_path = convert_path_format(path);
        let route_key = format!("{}-{}", method.to_uppercase(), path);

        let path_item = paths
            .entry(openapi_path.clone())
            .or_insert_with(|| json!({}));

        if let Some(path_obj) = path_item.as_object_mut() {
            let operation = if let Some(custom) = custom_info.get(&route_key) {
                // Use custom swagger info
                create_operation_from_custom(custom, path)
            } else {
                // Use default swagger info
                create_default_operation(method, path)
            };

            path_obj.insert(method.to_lowercase(), operation);
        }
    }

    let swagger_doc = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Ree API",
            "version": "1.0.0",
            "description": "API documentation generated by Ree framework"
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

    serde_json::to_string_pretty(&swagger_doc).unwrap()
}

/// Convert Ree path format to OpenAPI format
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

/// Create operation from custom swagger info
fn create_operation_from_custom(custom: &SwaggerInfo, path: &str) -> Value {
    let mut operation = json!({
        "summary": custom.summary,
        "description": custom.description,
        "tags": custom.tags,
    });

    // Add parameters
    let mut parameters = custom.parameters.clone();
    
    // Auto-detect path parameters if not explicitly defined
    for part in path.split('/') {
        if let Some(param_name) = part.strip_prefix(':') {
            if !parameters.iter().any(|p| p.name == param_name) {
                parameters.push(Parameter {
                    name: param_name.to_string(),
                    in_: "path".to_string(),
                    description: Some(format!("The {} parameter", param_name)),
                    required: true,
                    schema: Schema {
                        type_: "string".to_string(),
                        format: None,
                        example: None,
                        properties: None,
                        items: None,
                    },
                });
            }
        } else if let Some(param_name) = part.strip_prefix('*')
            && !parameters.iter().any(|p| p.name == param_name) {
                parameters.push(Parameter {
                    name: param_name.to_string(),
                    in_: "path".to_string(),
                    description: Some(format!("The {} wildcard parameter", param_name)),
                    required: true,
                    schema: Schema {
                        type_: "string".to_string(),
                        format: None,
                        example: None,
                        properties: None,
                        items: None,
                    },
                });
            }
    }

    if !parameters.is_empty() {
        operation["parameters"] = serde_json::to_value(parameters).unwrap();
    }

    // Add responses
    if !custom.responses.is_empty() {
        operation["responses"] = serde_json::to_value(&custom.responses).unwrap();
    } else {
        operation["responses"] = json!({
            "200": {
                "description": "Success"
            }
        });
    }

    // Add request body
    if let Some(request_body) = &custom.request_body {
        operation["requestBody"] = serde_json::to_value(request_body).unwrap();
    }

    // Add security
    if !custom.security.is_empty() {
        let security_array: Vec<Value> = custom
            .security
            .iter()
            .map(|req| {
                json!({
                    req.name.clone(): req.scopes
                })
            })
            .collect();
        operation["security"] = json!(security_array);
    }

    operation
}

/// Create default operation
fn create_default_operation(method: &str, path: &str) -> Value {
    let mut operation = json!({
        "summary": format!("{} {}", method, path),
        "responses": {
            "200": {
                "description": "Success"
            }
        }
    });

    // Extract path parameters
    let mut parameters = Vec::new();
    for part in path.split('/') {
        if let Some(param_name) = part.strip_prefix(':') {
            parameters.push(json!({
                "name": param_name,
                "in": "path",
                "required": true,
                "schema": {
                    "type": "string"
                },
                "description": format!("The {} parameter", param_name)
            }));
        } else if let Some(param_name) = part.strip_prefix('*') {
            parameters.push(json!({
                "name": param_name,
                "in": "path",
                "required": true,
                "schema": {
                    "type": "string"
                },
                "description": format!("The {} wildcard parameter", param_name)
            }));
        }
    }

    if !parameters.is_empty() {
        operation["parameters"] = json!(parameters);
    }

    // Add request body for POST, PUT, PATCH methods
    if matches!(method, "POST" | "PUT" | "PATCH") {
        operation["requestBody"] = json!({
            "required": true,
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object"
                    }
                }
            }
        });
    }

    operation
}

/// Generate Swagger UI HTML (unchanged)
pub fn generate_swagger_ui(json_url: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Ree API Documentation</title>
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