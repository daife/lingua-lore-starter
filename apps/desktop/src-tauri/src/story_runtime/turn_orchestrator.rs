use anyhow::{anyhow, Result};
use tokio::time::{sleep, Duration};

use crate::deepseek::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, DeepSeekClient, ResponseFormat,
};
use crate::domain::{ApiProfile, StoryTurnInput, StoryTurnPreview, StoryTurnResult, TurnOutput};
use crate::storage::AppState;
use crate::story_runtime::context_loader::load_context;
use crate::story_runtime::output_parser::parse_turn_output;
use crate::story_runtime::output_validator::validate_turn_output;
use crate::story_runtime::prompt_builder::{build_messages, TURN_OUTPUT_VALIDATION_RULES};
use crate::tool_runtime::{execute_readonly, read_only_tool_definitions};
use crate::turn_commit::committer::commit_turn;

const MAX_MODEL_REQUEST_ATTEMPTS: usize = 4;
const MAX_TURN_REPAIR_ATTEMPTS: usize = 4;
const MAX_TOOL_ROUNDS: usize = 3;
const MAX_TOOL_CALLS_TOTAL: usize = 8;

pub async fn preview_story_turn(
    state: &AppState,
    api_profile: ApiProfile,
    input: StoryTurnInput,
) -> Result<StoryTurnPreview> {
    let conn = state.open_world_conn(&input.world_id)?;
    let context = load_context(&conn, &input)?;
    let mut messages = build_messages(&context)?;
    let tools = read_only_tool_definitions();
    let client = DeepSeekClient::new(api_profile.clone());
    let mut tool_rounds = 0;
    let mut tool_calls_total = 0;
    let mut repair_attempts = 0;
    let mut last_validation_error = None;

    while repair_attempts < MAX_TURN_REPAIR_ATTEMPTS {
        let response = chat_completion_with_retries(
            &client,
            ChatCompletionRequest {
                model: api_profile.model.clone(),
                messages: messages.clone(),
                tools: Some(tools.clone()),
                tool_choice: Some("auto".to_string()),
                response_format: Some(ResponseFormat {
                    kind: "json_object".to_string(),
                }),
                temperature: 0.85,
                max_tokens: 4096,
                stream: false,
            },
        )
        .await?;

        let message = response
            .choices
            .first()
            .ok_or_else(|| anyhow!("DeepSeek returned no choices"))?
            .message
            .clone();

        if let Some(tool_calls) = message.tool_calls.clone() {
            tool_rounds += 1;
            tool_calls_total += tool_calls.len();
            if tool_rounds > MAX_TOOL_ROUNDS || tool_calls_total > MAX_TOOL_CALLS_TOTAL {
                messages.push(ChatMessage::user(
                    "Stop calling tools. Return the final valid json now.",
                ));
                continue;
            }
            messages.push(message);
            for call in tool_calls {
                let result =
                    execute_readonly(&conn, &call.function.name, &call.function.arguments)?;
                messages.push(ChatMessage::tool(call.id, serde_json::to_string(&result)?));
            }
            continue;
        }

        let content = message.content.unwrap_or_default();
        if content.trim().is_empty() {
            repair_attempts += 1;
            last_validation_error = Some("DeepSeek returned empty content".to_string());
            messages.push(ChatMessage::user(
                "Return the final result as valid json only. Do not return empty content.",
            ));
            continue;
        }

        match parse_turn_output(&content).and_then(|output| {
            validate_turn_output(&output)?;
            Ok(output)
        }) {
            Ok(output) => {
                return Ok(StoryTurnPreview {
                    input,
                    raw_output_json: content,
                    output,
                });
            }
            Err(err) => {
                repair_attempts += 1;
                last_validation_error = Some(err.to_string());
                messages.push(ChatMessage::user(format!(
                    "Your previous response was invalid json or failed validation: {err}. Return the same turn again as valid json only.\n\nValidation constraints:\n{TURN_OUTPUT_VALIDATION_RULES}"
                )));
            }
        }
    }

    Err(anyhow!(
        "DeepSeek could not produce a valid story turn after retries. Last error: {}",
        last_validation_error.unwrap_or_else(|| "unknown validation failure".to_string())
    ))
}

pub fn commit_story_turn_preview(
    state: &AppState,
    preview: StoryTurnPreview,
) -> Result<StoryTurnResult> {
    validate_turn_output(&preview.output)?;
    commit_story_turn_output(
        state,
        &preview.input,
        preview.output,
        &preview.raw_output_json,
    )
}

fn commit_story_turn_output(
    state: &AppState,
    input: &StoryTurnInput,
    output: TurnOutput,
    raw_output_json: &str,
) -> Result<StoryTurnResult> {
    let conn = state.open_world_conn(&input.world_id)?;
    commit_turn(&conn, input, output, raw_output_json)
}

async fn chat_completion_with_retries(
    client: &DeepSeekClient,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse> {
    let mut last_error = None;
    for attempt in 0..MAX_MODEL_REQUEST_ATTEMPTS {
        match client.chat_completion(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(err) => {
                last_error = Some(err);
                if attempt + 1 < MAX_MODEL_REQUEST_ATTEMPTS {
                    sleep(Duration::from_millis(400 * (attempt + 1) as u64)).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow!("DeepSeek request failed")))
}
