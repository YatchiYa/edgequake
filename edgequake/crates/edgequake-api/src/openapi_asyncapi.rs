//! Standalone AsyncAPI 2.6 document for WebSocket progress channels (SPEC-027 OAS-008 A++).
//!
//! SPEC-149: documents multiplexed subscribe/unsubscribe on `/ws/pipeline/progress`.

use serde_json::{json, Value};

/// Build the canonical AsyncAPI document served at `/api-docs/asyncapi.json`.
pub fn asyncapi_document() -> Value {
    json!({
        "asyncapi": "2.6.0",
        "info": {
            "title": "EdgeQuake WebSocket Progress",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Real-time pipeline and document processing progress over WebSocket (RFC 6455). SPEC-149: clients subscribe to owned track_ids on the global channel; events use a tagged {type,data} envelope."
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
                "description": "Multiplexed pipeline progress. Auth via ?token= or Authorization. Clients must send subscribe for track-scoped events.",
                "bindings": {
                    "ws": {
                        "method": "GET",
                        "query": {
                            "type": "object",
                            "properties": {
                                "token": {
                                    "type": "string",
                                    "description": "JWT or API key when auth enabled (browsers cannot set Authorization on WebSocket)"
                                }
                            }
                        },
                        "headers": {
                            "type": "object",
                            "properties": {
                                "Authorization": {
                                    "type": "string",
                                    "description": "Bearer JWT or X-API-Key when auth enabled (non-browser clients)"
                                }
                            }
                        }
                    }
                },
                "publish": {
                    "description": "Client commands (SPEC-149)",
                    "message": {
                        "oneOf": [
                            {
                                "name": "subscribe",
                                "payload": {
                                    "type": "object",
                                    "required": ["type", "track_ids"],
                                    "properties": {
                                        "type": { "const": "subscribe" },
                                        "track_ids": {
                                            "type": "array",
                                            "items": { "type": "string" }
                                        }
                                    },
                                    "example": {
                                        "type": "subscribe",
                                        "track_ids": ["f6fa9cad-bbff-4892-a855-3bd7d70da044"]
                                    }
                                }
                            },
                            {
                                "name": "unsubscribe",
                                "payload": {
                                    "type": "object",
                                    "required": ["type", "track_ids"],
                                    "properties": {
                                        "type": { "const": "unsubscribe" },
                                        "track_ids": {
                                            "type": "array",
                                            "items": { "type": "string" }
                                        }
                                    }
                                }
                            },
                            {
                                "name": "cancel",
                                "payload": {
                                    "type": "object",
                                    "required": ["type", "track_id"],
                                    "properties": {
                                        "type": { "const": "cancel" },
                                        "track_id": { "type": "string" }
                                    }
                                }
                            },
                            {
                                "name": "ping",
                                "payload": {
                                    "type": "object",
                                    "required": ["type"],
                                    "properties": {
                                        "type": { "const": "ping" },
                                        "client_time": { "type": "string" }
                                    }
                                }
                            }
                        ]
                    }
                },
                "subscribe": {
                    "message": {
                        "name": "ProgressEvent",
                        "payload": {
                            "oneOf": [
                                {
                                    "type": "object",
                                    "description": "Tagged ProgressEvent envelope",
                                    "required": ["type", "data"],
                                    "properties": {
                                        "type": {
                                            "type": "string",
                                            "enum": [
                                                "Connected",
                                                "Heartbeat",
                                                "StatusSnapshot",
                                                "StageTransition",
                                                "PdfPageProgress",
                                                "ChunkProgress",
                                                "ChunkFailure",
                                                "GraphStorageProgress",
                                                "SubscribedAck",
                                                "Message"
                                            ]
                                        },
                                        "data": { "type": "object" }
                                    },
                                    "example": {
                                        "type": "StageTransition",
                                        "data": {
                                            "document_id": "doc-1",
                                            "task_id": "f6fa9cad-bbff-4892-a855-3bd7d70da044",
                                            "stage": "extracting",
                                            "stage_message": "Extracting entities",
                                            "stage_progress": 0.4
                                        }
                                    }
                                },
                                {
                                    "type": "object",
                                    "description": "BulkDeletion* events (issue #309)",
                                    "properties": {
                                        "type": {
                                            "type": "string",
                                            "enum": [
                                                "BulkDeletionStarted",
                                                "BulkDeletionItemProgress",
                                                "BulkDeletionCompleted",
                                                "BulkDeletionFailed"
                                            ]
                                        },
                                        "data": {
                                            "type": "object",
                                            "properties": {
                                                "wipe_track_id": { "type": "string" },
                                                "workspace_id": { "type": "string" },
                                                "deleted_count": { "type": "integer" },
                                                "error_message": { "type": "string" }
                                            }
                                        }
                                    }
                                }
                            ]
                        }
                    }
                }
            },
            "/ws/progress/{track_id}": {
                "description": "Per-track document processing progress (compatibility endpoint)",
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
                                "type": "StageTransition",
                                "data": {
                                    "task_id": "f6fa9cad-bbff-4892-a855-3bd7d70da044",
                                    "document_id": "doc-1",
                                    "stage": "completed",
                                    "stage_message": "Processing complete",
                                    "stage_progress": 1.0
                                }
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
        let publish = &doc["channels"]["/ws/pipeline/progress"]["publish"];
        assert!(publish["message"]["oneOf"].is_array());
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
        assert!(bulk["properties"]["type"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e == "BulkDeletionFailed"));
    }

    #[test]
    fn spec149_documents_subscribe_command() {
        let doc = asyncapi_document();
        let one_of = doc["channels"]["/ws/pipeline/progress"]["publish"]["message"]["oneOf"]
            .as_array()
            .expect("publish oneOf");
        assert!(one_of.iter().any(|m| m["name"] == "subscribe"));
    }
}
