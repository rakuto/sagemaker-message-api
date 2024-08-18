use aws_sdk_sagemakerruntime::primitives::Blob;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct ChatCompletions {
    pub model: String,
    pub messages: Vec<ChatCompletionsMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i64>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub stream: Option<bool>,
    pub do_sample: Option<bool>,
    pub context: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatCompletionsMessage {
    pub role: String,
    pub content: String,
}

impl ChatCompletionsMessage {
    pub fn new<S: AsRef<str>>(role: S, content: S) -> ChatCompletionsMessage {
        ChatCompletionsMessage {
            role: role.as_ref().to_owned(),
            content: content.as_ref().to_owned(),
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionsResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<ChatCompletionsChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ChatCompletionsUsage>,
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionsChoice {
    pub index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<ChatCompletionsMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<ChatCompletionsChoiceDelta>,
    pub logprobs: Option<i32>,
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionsChoiceDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionsUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}


#[derive(Serialize, Debug, Default)]
pub struct PredictParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_new_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_sample: Option<bool>,
}

impl SMPredictionRequest {
    pub fn serialize(&self) -> Blob {
        Blob::new(json!(self).to_string().as_str())
    }
}

#[derive(Serialize, Debug, Default)]
pub struct SMPredictionRequest {
    pub inputs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<PredictParams>,
}

#[derive(Deserialize, Debug)]
pub struct SMPredictionOutput {
    pub generated_text: String,
}

#[derive(Serialize, Debug, Default)]
pub struct BedrockRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_gen_len: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

impl BedrockRequest {
    pub fn serialize(&self) -> Blob { Blob::new(json!(self).to_string().as_str()) }
}

#[derive(Deserialize, Debug, Default)]
pub struct BedrockResponse {
    pub generation: String,
    pub stop_reason: String,
}
