# Message API

Offers fully compatible with the OpenAI Chat Completion API for SageMaker Inference endpoints.

## Calling API with OpenAI Python library

```python
from openai import OpenAI

openai = OpenAI(base_url="https://chatapi.app.tne.ai/v1")

openai.chat.completions.create(
    max_tokens=500,
    model="llama-3-70B-instruct",
    messages=[
        {"role": "system", "content": "You are a pirate chatbot who always responds in pirate speak!"},
        {"role": "user", "content": "Can you introduce yourself?"},
    ]
)
```
