use anyhow::{anyhow, Result};
use reqwest::Client;

use crate::deepseek::types::{ChatCompletionRequest, ChatCompletionResponse};
use crate::domain::ApiProfile;

pub struct DeepSeekClient {
    http: Client,
    profile: ApiProfile,
}

impl DeepSeekClient {
    pub fn new(profile: ApiProfile) -> Self {
        Self {
            http: Client::new(),
            profile,
        }
    }

    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let base = self.profile.base_url.trim_end_matches('/');
        let url = format!("{base}/chat/completions");
        let response = self
            .http
            .post(url)
            .bearer_auth(&self.profile.api_key)
            .json(&request)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(anyhow!("DeepSeek request failed with {status}: {text}"));
        }
        serde_json::from_str(&text)
            .map_err(|err| anyhow!("invalid DeepSeek response: {err}; body={text}"))
    }
}
