# ai-game-master

## Auth

API больше не принимает `x-user-id`. Для всех защищённых эндпоинтов нужен
`Authorization: Bearer <jwt>`.

Поддержанный на текущий момент алгоритм подписи:
- `HS256`

Обязательные env-переменные для API:
- `JWT_SECRET`: общий секрет между IAM и этим сервисом.
- `JWT_ALGORITHM`: сейчас `HS256`.

Опциональные env-переменные:
- `JWT_ISSUER`: проверка claim `iss`.
- `JWT_AUDIENCE`: проверка claim `aud`.
- `JWT_USER_ID_CLAIM`: claim, из которого берётся UUID пользователя. По умолчанию `sub`.
- `JWT_LEEWAY_SECONDS`: допуск по времени для `exp`/`nbf`.

JWT должен содержать:
- `exp`
- claim пользователя (`sub` по умолчанию) со строковым UUID

Пример payload:

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "iam-service",
  "aud": "ai-game-master",
  "exp": 4102444800
}
```

## Character Backstory LLM

Бэкстори не генерируется отдельным endpoint-ом. Она создаётся внутри обычного
turn flow, когда `Narrator` сам вызывает `create_context_object` для объекта
типа `Npc`. В этом случае API обогащает `long_desc` этого `ContextObject` через
локальный OpenAI-compatible backend с Qwen перед сохранением объекта.

Поддержанные env-переменные:

- `BACKSTORY_LLM_BASE_URL`: base URL локального сервера, по умолчанию `http://127.0.0.1:8000/v1`
- `BACKSTORY_LLM_API_KEY`: опциональный bearer token для совместимых серверов
- `BACKSTORY_LLM_MODEL`: имя модели, по умолчанию `Qwen/Qwen2.5-7B-Instruct`
- `BACKSTORY_LLM_TEMPERATURE`: температура, по умолчанию `0.75`
- `BACKSTORY_LLM_MAX_TOKENS`: лимит ответа, по умолчанию `450`

Если API запущен в Docker, а модель работает на хост-машине, используйте:

- `BACKSTORY_LLM_BASE_URL=http://host.docker.internal:8000/v1`

`docker-compose.yml` уже добавляет `host.docker.internal -> host-gateway`, чтобы
контейнер API видел модель, запущенную на вашей машине.
