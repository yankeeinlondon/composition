# AI Models

We want callers to be able to express what models (and providers) they would like to use while ensuring that appropriate API Keys are available.

- the providers we will support are:
    - OpenAI
    - Anthropic
    - Gemini
    - Open Router
    - Ollama (for local models)
- the "nice-to-have" providers we want to support are:
    - [Kimi K2](https://kimi.com)
    - [Zai](https://z.ai) (aka, GLM models)
    - [Deepseek](deepseek.com)
    - [ZenMux](https://zenmux.ai/) - an aggregator similar to Open Router (but less expensive)
- the syntax we will use to express an LLM model will be `${provider}/${model}`
- we need to have a sustainable way of producing a valid provider to model mapping
    - Ideally we'd find a way to get this as a JSON resource somewhere and then we could refresh it very easily (ideally exposed via a library call)
- for all providers supported we'll want a mapping to:
    - the ENV variable we'd expect their API Key to be found in
    - the BaseURL if they use the OpenAI API standard

## Use Cases

We will be using AI for the following:

1. Summarization - we will receive Markdown, HTML, or PDF content and we will want to be able to summarize the contents.
2. Consolidation - we will receive multiple documents and will want the AI to consolidate all the knowledge into a single document.
3. Topic Extraction - from a body of documents we want to extract a single topic and then structure it as a well-formed document based on the topic
4. Vector Embeddings - both Encode and Decode


## Model Selection via ENV Variables

All ENV variables are expected to be in the `${provider}/${model}` format

- `MODEL_FAST` - will be used for AI uses cases that benefit more from speed than from having the fastest model
- `MODEL` - will be used for most uses cases except for vector embeddings
- `LOCAL_FALLBACK` - will be used when user is offline; is expected to be an Ollama model
    - Note: this does not preclude you from using local models for `MODEL`, `MODEL_FAST`, or `MODEL_STRONG`
- `MODEL_STRONG` - will be used for tasks which require more heavy lifting
- `EMBEDDINGS` - will be the default embeddings model used for encoding and decoding

## Model Selection via Frontmatter

When the `render()` function is called on a Markdown/Darkmatter file we have the chance to pass in an initial configuration/state that will represent the baseline frontmatter throughout the rendering pipeline.

This means that we can call parse and add the following key/values in as frontmatter and this will override any ENV settings where there is a conflict:

- `model_fast`
- `model`
- `local_fallback`
- `model_strong`
- `embeddings`

**Note:** individual pages can override any frontmatter fit but while it is considered a bad idea to set these properties, you can use the `use_model` property and set it to `normal`, `fast`, or `strong` and this will push all tasks to use the kind of model you want on that page.
