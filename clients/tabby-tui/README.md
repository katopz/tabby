# tabby-tui

A [TabbyML](https://github.com/tabbyml) terminal ui on client side with [Rataui](https://github.com/ratatui-org/ratatui). Will require [TabbyML](https://github.com/tabbyml) service endpoint to be running on the local (or any) server.

## Prerequisites

> Serving endpoint.

```bash
sudo docker run -it --gpus all -p 8080:8080 -v $HOME/.tabby:/data tabbyml/tabby serve --model TabbyML/Mistral-7B --device cuda
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
