# SageMaker Message API Proxy

High-performance proxy server offering OpenAI Chat Completion compatible API for
SageMaker Inference endpoints.

## Configure SageMaker endpoints

```yaml
models:
  - model: Llama-3-8B
    endpoint_name: inference-component-endpoint
    inference_component: llama-3-8b
    backend: LMI
  - model: Llama-3-70B-Instruct
    endpoint_name: inference-component-endpoint
    inference_component: llama-3-70b-instruct
    backend: LMI
  - model: Phi-3-medium-4k-instruct
    endpoint_name: inference-component-endpoint
    inference_component: phi-3-medium-4k
    backend: LMI
  - model: Llama3-ChatQA-1.5-8B
    endpoint_name: inference-component-endpoint
    inference_component: llama-3-chatqa-8b
    backend: LMI
```

## Calling API with OpenAI Python library

```python
from openai import OpenAI

openai = OpenAI(base_url="http://localhost:8900/v1")

openai.chat.completions.create(
    max_tokens=500,
    model="llama-3-70B-instruct",
    messages=[
        {"role": "system", "content": "You are a pirate chatbot who always responds in pirate speak!"},
        {"role": "user", "content": "Can you introduce yourself?"},
    ]
)
```
