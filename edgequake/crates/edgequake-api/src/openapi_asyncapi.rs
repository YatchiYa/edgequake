//! Standalone AsyncAPI 2.6 document for WebSocket progress channels (SPEC-027 OAS-008 A++).

use serde_json::{json, Value};

/// Build the canonical AsyncAPI document served at `/api-docs/asyncapi.json`.
pub fn asyncapi_document() -> Value {
    json!({
        "asyncapi": "2.6.0",
        "info": {
            "title": "EdgeQuake WebSocket Progress",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Real-time pipeline and document processing progress over WebSocket (RFC 6455)."
        },
        "defaultContentType": "application/json",
        "servers": {
            "local": {
                "url": "ws://localhost:8080",
                "protocol": "ws",
                "description": "Local development backend"
            }
        },
        "channels": {
            "/ws/pipeline/progress": {
                "description": "Global pipeline progress stream (all tracks in workspace context)",
                "bindings": {
                    "ws": {
                        "method": "GET",
                        "headers": {
                            "type": "object",
                            "properties": {
                                "Authorization": {
                                    "type": "string",
                                    "description": "Bearer JWT or X-API-Key when auth enabled"
                                }
                            }
                        }
                    }
                },
                "subscribe": {
                    "message": {
                        "name": "ProgressEvent",
                        "payload": {
                            "oneOf": [
                                {
                                    "type": "object",
                                    "description": "Pipeline track progress",
                                    "properties": {
                                        "track_id": { "type": "string" },
                                        "phase": { "type": "string" },
                                        "progress": { "type": "number", "minimum": 0, "maximum": 100 },
                                        "message": { "type": "string" }
                                    }
                                },
                                {
                                    "type": "object",
                                    "description": "BulkDeletion* events (issue #309)",
                                    "properties": {
                                        "event": {
                                            "type": "string",
                                            "enum": [
                                                "BulkDeletionStarted",
                                                "BulkDeletionItemProgress",
                                                "BulkDeletionCompleted",
                                                "BulkDeletionFailed"
                                            ]
                                        },
                                        "wipe_track_id": { "type": "string" },
                                        "workspace_id": { "type": "string" },
                                        "deleted_count": { "type": "integer" },
                                        "error_message": { "type": "string" }
                                    },
                                    "required": ["wipe_track_id"]
                                }
                            ],
                            "example": {
                                "event": "BulkDeletionFailed",
                                "wipe_track_id": "workspace_wipe-f6fa9cad",
                                "workspace_id": "940fadab-2390-4b29-af7e-ff27fd6d7755",
                                "error_message": "workspace wipe graph clear failed"
                            }
                        }
                    }
                }
            },
            "/ws/progress/{track_id}": {
                "description": "Per-track document processing progress",
                "parameters": {
                    "track_id": {
                        "description": "Document track UUID",
                        "schema": { "type": "string", "format": "uuid" }
                    }
                },
                "bindings": {
                    "ws": { "method": "GET" }
                },
                "subscribe": {
                    "message": {
                        "name": "TrackProgressEvent",
                        "payload": {
                            "type": "object",
                            "example": {
                                "track_id": "f6fa9cad-bbff-4892-a855-3bd7d70da044",
                                "phase": "completed",
                                "progress": 100.0,
                                "message": "Processing complete"
                            }
                        }
                    }
                }
            }
        }
    })
}

/// Sidecar fragment embedded in OpenAPI root (`x-edgequake-asyncapi`).
pub fn asyncapi_sidecar() -> Value {
    asyncapi_document()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_has_required_channels() {
        let doc = asyncapi_document();
        assert_eq!(doc["asyncapi"], "2.6.0");
        assert!(doc["channels"]["/ws/pipeline/progress"].is_object());
        assert!(doc["channels"]["/ws/progress/{track_id}"].is_object());
        let payload = &doc["channels"]["/ws/pipeline/progress"]["subscribe"]["message"]["payload"];
        let one_of = payload["oneOf"].as_array().expect("oneOf bulk+pipeline");
        let bulk = one_of
            .iter()
            .find(|v| {
                v["description"]
                    .as_str()
                    .unwrap_or("")
                    .contains("BulkDeletion")
            })
            .expect("BulkDeletion schema");
        assert!(bulk["properties"]["wipe_track_id"].is_object());
        let events = bulk["properties"]["event"]["enum"]
            .as_array()
            .expect("event enum");
        assert!(events.iter().any(|e| e == "BulkDeletionFailed"));
    }
}
