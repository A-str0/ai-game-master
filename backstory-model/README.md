# D&D Backstory Generator API

Короткое описание

This is a small REST API that generates D&D character backstories using a fine-tuned Qwen2.5-0.5B model with LoRA adapters.

Endpoints

- `GET /health` — health check
- `POST /generate` — generate a single backstory (`name`, `race`, `class`, optional `max_new_tokens`)
- `POST /batch` — generate multiple backstories (`characters` array)

Run with Docker Compose

From the repository root run:

```bash
docker compose -f backstory-model/docker-compose.yml up --build
```

This will start the API and mount the model artifacts found under `backstory-model/artifacts/finetuned`.

When running the full project from the repository root, the root `docker-compose.yml`
starts this service as `backstory-model` and the Rust API calls `http://backstory-model:5000/generate`.

License: MIT
