---
title: Structured Outputs
subtitle: Return structured data from your models
headline: Structured Outputs | Enforce JSON Schema in OpenRouter API Responses
canonical-url: 'https://openrouter.ai/docs/features/structured-outputs'
'og:site_name': OpenRouter Documentation
'og:title': Structured Outputs - Type-Safe JSON Responses from AI Models
'og:description': >-
  Enforce JSON Schema validation on AI model responses. Get consistent,
  type-safe outputs and avoid parsing errors with OpenRouter's structured output
  feature.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=Structured%20Outputs&description=Type-Safe%20JSON%20Responses%20from%20AI%20Models
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

import { API_KEY_REF } from '../../../imports/constants';

OpenRouter supports structured outputs for compatible models, ensuring responses follow a specific JSON Schema format. This feature is particularly useful when you need consistent, well-formatted responses that can be reliably parsed by your application.

## Overview

Structured outputs allow you to:

- Enforce specific JSON Schema validation on model responses
- Get consistent, type-safe outputs
- Avoid parsing errors and hallucinated fields
- Simplify response handling in your application

## Using Structured Outputs

To use structured outputs, include a `response_format` parameter in your request, with `type` set to `json_schema` and the `json_schema` object containing your schema:

```typescript
{
  "messages": [
    { "role": "user", "content": "What's the weather like in London?" }
  ],
  "response_format": {
    "type": "json_schema",
    "json_schema": {
      "name": "weather",
      "strict": true,
      "schema": {
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "City or location name"
          },
          "temperature": {
            "type": "number",
            "description": "Temperature in Celsius"
          },
          "conditions": {
            "type": "string",
            "description": "Weather conditions description"
          }
        },
        "required": ["location", "temperature", "conditions"],
        "additionalProperties": false
      }
    }
  }
}
```

The model will respond with a JSON object that strictly follows your schema:

```json
{
  "location": "London",
  "temperature": 18,
  "conditions": "Partly cloudy with light drizzle"
}
```

## Model Support

Structured outputs are supported by select models.

You can find a list of models that support structured outputs on the [models page](https://openrouter.ai/models?order=newest&supported_parameters=structured_outputs).

- OpenAI models (GPT-4o and later versions) [Docs](https://platform.openai.com/docs/guides/structured-outputs)
- All Fireworks provided models [Docs](https://docs.fireworks.ai/structured-responses/structured-response-formatting#structured-response-modes)

To ensure your chosen model supports structured outputs:

1. Check the model's supported parameters on the [models page](https://openrouter.ai/models)
2. Set `require_parameters: true` in your provider preferences (see [Provider Routing](/docs/features/provider-routing))
3. Include `response_format` and set `type: json_schema` in the required parameters

## Best Practices

1. **Include descriptions**: Add clear descriptions to your schema properties to guide the model

2. **Use strict mode**: Always set `strict: true` to ensure the model follows your schema exactly

## Example Implementation

Here's a complete example using the Fetch API:

<Template data={{
  API_KEY_REF,
  MODEL: 'openai/gpt-4'
}}>
<CodeGroup>

```typescript title="With TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
  method: 'POST',
  headers: {
    Authorization: 'Bearer {{API_KEY_REF}}',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: '{{MODEL}}',
    messages: [
      { role: 'user', content: 'What is the weather like in London?' },
    ],
    response_format: {
      type: 'json_schema',
      json_schema: {
        name: 'weather',
        strict: true,
        schema: {
          type: 'object',
          properties: {
            location: {
              type: 'string',
              description: 'City or location name',
            },
            temperature: {
              type: 'number',
              description: 'Temperature in Celsius',
            },
            conditions: {
              type: 'string',
              description: 'Weather conditions description',
            },
          },
          required: ['location', 'temperature', 'conditions'],
          additionalProperties: false,
        },
      },
    },
  }),
});

const data = await response.json();
const weatherInfo = data.choices[0].message.content;
```

```python title="With Python"
import requests
import json

response = requests.post(
  "https://openrouter.ai/api/v1/chat/completions",
  headers={
    "Authorization": f"Bearer {{API_KEY_REF}}",
    "Content-Type": "application/json",
  },

  json={
    "model": "{{MODEL}}",
    "messages": [
      {"role": "user", "content": "What is the weather like in London?"},
    ],
    "response_format": {
      "type": "json_schema",
      "json_schema": {
        "name": "weather",
        "strict": True,
        "schema": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "City or location name",
            },
            "temperature": {
              "type": "number",
              "description": "Temperature in Celsius",
            },
            "conditions": {
              "type": "string",
              "description": "Weather conditions description",
            },
          },
          "required": ["location", "temperature", "conditions"],
          "additionalProperties": False,
        },
      },
    },
  },
)

data = response.json()
weather_info = data["choices"][0]["message"]["content"]
```

</CodeGroup>

</Template>

## Streaming with Structured Outputs

Structured outputs are also supported with streaming responses. The model will stream valid partial JSON that, when complete, forms a valid response matching your schema.

To enable streaming with structured outputs, simply add `stream: true` to your request:

```typescript
{
  "stream": true,
  "response_format": {
    "type": "json_schema",
    // ... rest of your schema
  }
}
```

## Error Handling

When using structured outputs, you may encounter these scenarios:

1. **Model doesn't support structured outputs**: The request will fail with an error indicating lack of support
2. **Invalid schema**: The model will return an error if your JSON Schema is invalid
