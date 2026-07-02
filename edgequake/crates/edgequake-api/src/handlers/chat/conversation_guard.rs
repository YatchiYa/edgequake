//! Conversation existence checks before persisting assistant messages (SPEC-040 #259).

use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

/// Returns false when the conversation row was deleted (e.g. during a long stream).
pub async fn conversation_exists(state: &AppState, conversation_id: Uuid) -> ApiResult<bool> {
    Ok(state
        .conversation_service
        .get_conversation(conversation_id)
        .await?
        .is_some())
}
