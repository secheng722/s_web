//! Simple Swagger generation for Ree framework

use serde_json::json;

/// Generate OpenAPI JSON from routes
pub fn generate_swagger_json(routes: &[(String, String)]) -> String {
    let mut paths = serde_json::Map::new();
    
    for (method, path) in routes {
        // Convert Ree path format (:id) to OpenAPI format ({id})
        let openapi_path = path.replace(":", "{").replace("}", "");
        let openapi_path = if openapi_path.contains("{") && !openapi_path.contains("}") {
            // Add closing braces for parameters
            openapi_path.split('/').map(|part| {
                if part.starts_with("{") && !part.ends_with("}") {
                    format!("{part}}}")
                } else {
                    part.to_string()
                }
            }).collect::<Vec<_>>().join("/")
        } else {
            openapi_path
        };
        
        let path_item = paths.entry(openapi_path.clone()).or_insert_with(|| json!({}));
        
        if let Some(path_obj) = path_item.as_object_mut() {
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
                        "description": format!("The {param_name} parameter")
                    }));
                }
            }

            if !parameters.is_empty() {
                operation["parameters"] = json!(parameters);
            }

            // Add request body for POST, PUT, PATCH methods
            if matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
                operation["requestBody"] = json!({
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "name": {
                                        "type": "string",
                                        "example": "John Doe"
                                    },
                                    "email": {
                                        "type": "string",
                                        "format": "email",
                                        "example": "john@example.com"
                                    },
                                    "age": {
                                        "type": "integer",
                                        "example": 30
                                    }
                                }
                            },
                            "example": {
                                "name": "John Doe",
                                "email": "john@example.com",
                                "age": 30
                            }
                        }
                    }
                });
            }

            path_obj.insert(method.to_lowercase(), operation);
        }
    }
    
    let swagger_doc = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Ree API",
            "version": "1.0.0"
        },
        "paths": paths
    });
    
    serde_json::to_string_pretty(&swagger_doc).unwrap()
}

/// Generate Swagger UI HTML
pub fn generate_swagger_ui(json_url: &str) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Ree API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
    <style>
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
            deepLinking: true,
            showExtensions: true,
            showCommonExtensions: true,
            tryItOutEnabled: true,
            supportedSubmitMethods: ['get', 'post', 'put', 'delete', 'patch', 'head', 'options'],
            onComplete: function() {{
                console.log('Swagger UI loaded');
            }},
            requestInterceptor: function(request) {{
                console.log('Request:', request);
                return request;
            }},
            responseInterceptor: function(response) {{
                console.log('Response:', response);
                return response;
            }},
            docExpansion: 'list',
            filter: true,
            showRequestHeaders: true
        }});
    </script>
</body>
</html>
    "#)
}
