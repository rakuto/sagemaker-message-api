use anyhow::{anyhow, Result};

use crate::types::ChatCompletionsMessage;

pub fn apply_chat_template_llama3(messages: &[ChatCompletionsMessage]) -> String {
    let mut s = String::new();
    s.push_str("<|begin_of_text|>");
    for m in messages {
        s.push_str("<|start_header_id|>");
        s.push_str(&m.role);
        s.push_str("<|end_header_id|>");
        s.push_str("\n\n");
        s.push_str(&m.content);
        s.push_str("<|eot_id|>")
    }
    s.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
    s
}

pub fn apply_chat_template_phi3(messages: &[ChatCompletionsMessage]) -> String {
    let mut s = String::new();
    for m in messages {
        if m.role.as_str() == "user" || m.role == "system" {
            s.push_str("<|user|>\n");
        } else if m.role.as_str() == "assistant" {
            s.push_str("<|assistant|>\n")
        } else {
            continue;
        }
        s.push_str(m.content.as_str());
        s.push_str("<|end|>\n<|assistant|>\n");
    }
    s
}

/// Format prompt for nvidia/Llama3-ChatQA-1.5-8B
/// https://huggingface.co/nvidia/Llama3-ChatQA-1.5-8B
pub fn apply_chat_template_nvidia_llama3_chatqa<S: AsRef<str>>(
    messages: &[ChatCompletionsMessage],
    context: Option<S>) -> Result<String> {
    let mut s = String::new();

    for m in messages {
        match m.role.as_str() {
            "system" => {
                s.push_str("System: ");
                s.push_str(m.content.as_str());
                s.push_str("\n\n");
                if let Some(context) = context.as_ref() {
                    let context = context.as_ref();
                    s.push_str(context);
                    s.push_str("\n\n");
                }
            }
            "user" => {
                s.push_str("User: ");
                s.push_str(m.content.as_str());
                s.push_str("\n\n");
            }
            "assistant" => {
                s.push_str("Assistant: ");
                s.push_str(m.content.as_str());
                s.push_str("\n\n");
            }
            role => return Err(anyhow!(format!("unknown role: {}", role)))
        }
    }

    s.push_str("Assistant: ");

    Ok(s)
}

pub fn apply_chat_template<S>(
    model: S,
    messages: &[ChatCompletionsMessage],
    context: Option<String>) -> Result<String>
where
    S: AsRef<str>,
{
    if model.as_ref().to_lowercase().starts_with("llama-3") {
        Ok(apply_chat_template_llama3(messages))
    } else if model.as_ref().to_lowercase().starts_with("phi-3") {
        Ok(apply_chat_template_phi3(messages))
    } else if model.as_ref().to_lowercase() == "llama3-chatqa-1.5-8b" {
        apply_chat_template_nvidia_llama3_chatqa(messages, context)
    } else {
        Err(anyhow!(format!("Unknown model {}", model.as_ref())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_chat_template_llama3() {
        let mut messages = &[
            ChatCompletionsMessage::new("system", "You are a pirate chatbot who always responds in pirate speak!"),
            ChatCompletionsMessage::new("user", "Who are you?"),
        ];
        let mut expected = "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\n\
            You are a pirate chatbot who always responds in pirate speak!<|eot_id|>\
            <|start_header_id|>user<|end_header_id|>\n\n\
            Who are you?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n".to_owned();

        assert_eq!(apply_chat_template_llama3(messages), expected);
    }

    #[test]
    fn test_apply_chat_template_nvidia_llama3_chatqa() {
        let messages = &[
            ChatCompletionsMessage::new("system", "You are artificial intelligence assistant."),
            ChatCompletionsMessage::new("user", "Hello. What can you help?"),
        ];
        let context = "THIS IS CONTEXT";
        let expected = "\
System: You are artificial intelligence assistant.

THIS IS CONTEXT

User: Hello. What can you help?

Assistant: ";

        assert_eq!(apply_chat_template_nvidia_llama3_chatqa(messages, Some(context)).unwrap(), expected);

        let context: Option<String> = None;
        let expected = "\
System: You are artificial intelligence assistant.

User: Hello. What can you help?

Assistant: ";

        assert_eq!(apply_chat_template_nvidia_llama3_chatqa(messages, context.clone()).unwrap(), expected);

        // Multiturn chat
        let messages = &[
            ChatCompletionsMessage::new("system", "You are artificial intelligence assistant."),
            ChatCompletionsMessage::new("user", "Hello. What can you help?"),
            ChatCompletionsMessage::new("assistant", "I'm an AI assistant, and I'd be happy to help with a wide range of tasks."),
            ChatCompletionsMessage::new("user", "Nice. What else can you do?"),
        ];
        let expected = "\
System: You are artificial intelligence assistant.

User: Hello. What can you help?

Assistant: I'm an AI assistant, and I'd be happy to help with a wide range of tasks.

User: Nice. What else can you do?

Assistant: ";
        assert_eq!(apply_chat_template_nvidia_llama3_chatqa(messages, context.clone()).unwrap(), expected);
    }
}
