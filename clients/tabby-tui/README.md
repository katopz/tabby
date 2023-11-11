# tabby-tui

A [TabbyML](https://github.com/tabbyml) terminal ui on client side with [Rataui](https://github.com/ratatui-org/ratatui). Will require [TabbyML](https://github.com/tabbyml) service endpoint to be running on the local (or any) server.

## Prerequisites

> Serving endpoint.

```bash
# completions
docker run -it --gpus all -p 8080:8080 -v $HOME/.tabby:/data tabby_cuda12_2 serve --model TabbyML/Mistral-7B --device cuda

# completions + chat
docker run -it --gpus all -p 8080:8080 -v $HOME/.tabby:/data tabbyml/tabby:nightly serve --model TabbyML/StarCoder-1B --device cuda --chat-model TabbyML/Mistral-7B
```

## Dev

> Terminal UI

```bash
cargo watch -w src -x run
```

## TODO

- tabbyml-rs-sdk: For consume tabby services.
- CI/CD
  ```
  [![CI](https://github.com/tabby-tui/workflows/CI/badge.svg)](https://github.com/tabby-tui/actions)
  ```
- New build
  ```
  docker build -t local/llama.cpp:full-cuda -f .devops/full-cuda.Dockerfile .
  ```