extern crate core;

use std::fmt::Error;
use std::io::Write;
use std::pin::Pin;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::bail;
use async_stream::stream as async_stream;
use aws_sdk_sagemakerruntime as sagemakerruntime;
use aws_sdk_sagemakerruntime::primitives::Blob;
use axum::{
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    Json,
    response::IntoResponse,
    response::sse::{Event, KeepAlive, Sse},
    Router,
    routing::{get, post},
};
use bytes::{Buf, BufMut, BytesMut};
use clap::builder::Str;
use clap::Parser;
use futures::FutureExt;
use futures::stream::{self as stream, Stream};
use futures::task::Poll;
use futures_util::{pin_mut, StreamExt};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, error_span, event, info, info_span, instrument, Level, warn, warn_span};
use tracing_subscriber;
use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use uuid::Uuid;

use crate::chat_template::{apply_chat_template, apply_chat_template_llama3};
use crate::endpoint_loader::EndpointLoader;
use crate::types::{ChatCompletions, ChatCompletionsChoice, ChatCompletionsChoiceDelta, ChatCompletionsMessage, ChatCompletionsResponse, ChatCompletionsUsage, PredictionOutput, PredictParams, PredictRequest};

mod chat_template;
mod types;
mod sagemaker_endpoint_loader;
mod endpoint_loader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// HTTP server address
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// HTTP server port
    #[arg(short, long, default_value_t = 8900)]
    port: u16,

    /// A path to SageMaker inference endpoints config file.
    #[arg(short, long)]
    config: String,
}

#[derive(Clone, Debug)]
struct AppState {
    smr_client: Arc<sagemakerruntime::Client>,
    endpoints: Arc<EndpointLoader>,
}


async fn health() -> &'static str {
    "ok"
}

#[tracing::instrument]
async fn chat_completions(
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletions>,
) -> impl IntoResponse {
    let req_id = Uuid::new_v4();
    let span = info_span!("Start Chat completion");
    let _ = span.enter();
    let endpoint = match state.endpoints.get_endpoint(&payload.model) {
        Some(endpoint) => endpoint,
        None => return (StatusCode::BAD_REQUEST, "Unsupported model").into_response(),
    };
    let content_type = "application/json".to_owned();
    let prompt = if payload.model.to_lowercase().contains("-instruct") ||
        payload.model.eq("Llama3-ChatQA-1.5-8B") {
        apply_chat_template(&payload.model, &payload.messages, None).unwrap()
    } else {
        payload.messages.iter().map(|m| m.content.to_owned()).collect::<Vec<String>>().join("\n")
    };

    let created = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let body = PredictRequest {
        inputs: prompt,
        parameters: Some(PredictParams {
            top_p: payload.top_p,
            top_k: payload.top_k,
            temperature: payload.temperature,
            max_new_tokens: payload.max_tokens,
            do_sample: payload.do_sample,
        }),
    }.serialize();

    // EOS token
    let eot = if payload.model.starts_with("Llama") {
        "<|eot_id|>"
    } else if payload.model.starts_with("Phi-3") {
        "<|end|>"
    } else {
        return (StatusCode::BAD_REQUEST, "Unsupported model").into_response();
    };

    if payload.stream.unwrap_or(false) {
        let mut output = state.smr_client.invoke_endpoint_with_response_stream()
            .set_inference_id(Some(req_id.to_string()))
            .set_endpoint_name(Some(endpoint.endpoint_name.to_owned()))
            .set_inference_component_name(
                if let Some(inference_component) = &endpoint.inference_component {
                    Some(inference_component.to_owned())
                } else {
                    None
                }
            )
            .set_body(Some(body))
            .set_content_type(Some(content_type))
            .send()
            .await
            .unwrap();


        let mut stream_response = Box::pin(async_stream! {
            let start_seq = "{\"generated_text\": \"";
            let stop_seq = "\"}";

            let mut buf = BytesMut::new();
            loop {
                match output.body.recv().await {
                    Ok(Some(response_stream)) => {
                        let payload_part = response_stream.as_payload_part().unwrap();
                        let payload_blob = payload_part.bytes.as_ref().unwrap().as_ref();
                        buf.put(payload_blob);

                        if buf.starts_with(start_seq.as_bytes()) {
                            buf.advance(start_seq.len());
                        } else if buf.ends_with(stop_seq.as_bytes()) {
                            unsafe {
                                buf.set_len(buf.len() - stop_seq.as_bytes().len());
                            }
                        }
                        let chunk = buf.chunk();
                        let content = String::from_utf8(chunk.to_vec()).expect("payload is not UTF-8");
                        yield Ok(Some(content));
                        buf.advance(chunk.len());
                    }
                    Ok(None) => yield Ok(None),
                    Err(err) => yield Err(err),
                }
            }
        });

        let object = "chat.completion.chunk";
        let mut first_response = true;
        let mut done = false;
        let stream_responder = Box::pin(async_stream! {
            while let Some(Ok(mut chunk)) = stream_response.next().await {
                let mut finish_reason: Option<String> = None;
                if chunk.as_ref().is_some() {
                    if let Some(end_pos) = chunk.as_ref().unwrap().find(eot) {
                        chunk = Some(chunk.as_ref().unwrap()[0..end_pos].to_owned());
                        finish_reason = Some("stop".to_owned());
                        done = true;
                    }
                } else {
                    finish_reason = Some("length".to_owned());
                    done = true;
                }
                let role = if first_response {
                    first_response = false;
                    Some("assistant".to_owned())
                } else {
                    None
                };

                let data = ChatCompletionsResponse {
                    id: req_id.to_string(),
                    object: object.to_owned(),
                    created,
                    model: payload.model.to_owned(),
                    system_fingerprint: None,
                    choices: vec![
                        ChatCompletionsChoice {
                            index: 0,
                            message: None,
                            delta: Some(ChatCompletionsChoiceDelta {
                                role,
                                content: chunk,
                            }),
                            logprobs: None,
                            finish_reason,
                        }
                    ],
                    usage: None,
                };

                let event: Result<Event, Error> = Ok(Event::default().json_data(data).unwrap());
                yield event;

                if done {
                    break;
                }
            }
        });

        Sse::new(stream_responder)
            .keep_alive(KeepAlive::default())
            .into_response()
    } else {
        let output = state.smr_client.invoke_endpoint()
            .set_inference_id(Some(req_id.to_string()))
            .set_endpoint_name(Some(endpoint.endpoint_name.to_owned()))
            .set_inference_component_name(
                if let Some(inference_component) = &endpoint.inference_component {
                    Some(inference_component.to_owned())
                } else {
                    None
                }
            )
            .set_target_model(
                if let Some(target_model) = &endpoint.target_model {
                    Some(target_model.to_owned())
                } else {
                    None
                }
            )
            .set_body(Some(body))
            .set_content_type(Some(content_type))
            .send()
            .await
            .expect("invoke error");

        let predict_output: PredictionOutput = serde_json::from_slice(output.body.unwrap().as_ref()).unwrap();
        let eot_pos = predict_output.generated_text.find(eot);

        let mut finish_reason: String;
        let mut assistant_output: String;
        if let Some(pos) = eot_pos {
            finish_reason = "stop".to_owned();
            assistant_output = predict_output.generated_text[0..pos].to_owned()
        } else {
            finish_reason = "length".to_owned();
            assistant_output = predict_output.generated_text;
        };

        let output = ChatCompletionsResponse {
            id: req_id.to_string(),
            object: "chat.completion".to_owned(),
            created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            model: payload.model.to_owned(),
            choices: vec![
                ChatCompletionsChoice {
                    index: 0,
                    message: Some(ChatCompletionsMessage {
                        role: "assistant".to_owned(),
                        content: assistant_output,
                    }),
                    delta: None,
                    finish_reason: Some(finish_reason),
                    logprobs: None,
                }
            ],
            system_fingerprint: None,
            usage: Some(ChatCompletionsUsage::default()),
        };

        Json(output).into_response()
    }
}

#[tokio::main]
async fn main() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("could not create OTLP tracer");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .init();

    let args = Args::parse();
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(AppState {
            smr_client: Arc::new(sagemakerruntime::Client::new(&config)),
            endpoints: Arc::new(EndpointLoader::load(args.config).expect("unable to load config file")),
        })
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_headers(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS]),
        );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.address, args.port)).await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
