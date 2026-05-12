# ai-game-master

## Docker Compose

Единый запуск из корня репозитория:

```bash
docker compose up --build
```

Поднимаются:
- `iam-service` на `http://localhost:8081`
- `api` на `http://localhost:3000`
- общий `postgres`
- `qdrant`

`iam-service` не поднимает отдельную БД: он использует тот же контейнер
`postgres` и ту же базу из `POSTGRES_DB`.

`backstory-model` в этом compose не поднимается: core API обращается к внешнему
REST endpoint из `BACKSTORY_MODEL_BASE_URL`.

Core API подключается к Qdrant по gRPC через `QDRANT_GRPC_URL`
(`http://qdrant:6334` внутри Docker). REST-порт Qdrant `6333` нужен только для
ручной отладки с хоста.

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
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "iam-service",
  "aud": "ai-game-master",
  "exp": 4102444800
}
```

В обычном сценарии токен не нужно собирать руками. Получите его через IAM:

```bash
curl -X POST http://localhost:8081/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"123456"}'

curl -X POST http://localhost:8081/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"123456"}'
```

`/login` вернёт JSON вида `{"token":"<jwt>"}`. В запросы к core API передавайте
только этот JWT в HTTP-заголовке:

```http
Authorization: Bearer <jwt>
```

JWT подписывается алгоритмом `HS256`. `JWT_SECRET`, `JWT_ISSUER` и
`JWT_AUDIENCE` в `docker-compose.yml` передаются одновременно в `iam-service` и
в core API, поэтому токен из `/login` принимается core API без дополнительной
конвертации.

## Character Backstory LLM

Бэкстори не генерируется отдельным endpoint-ом. Она создаётся внутри обычного
turn flow, когда агент создаёт `ContextObject` типа `Npc`. В этом случае API
обогащает `long_desc` через внешний REST endpoint `backstory-model` перед
сохранением объекта.

Поддержанные env-переменные:

- `BACKSTORY_MODEL_BASE_URL`: base URL сервиса `backstory-model`, по умолчанию `https://kmfj59fk-5000.euw.devtunnels.ms`
- `BACKSTORY_MODEL_MAX_NEW_TOKENS`: лимит генерации, по умолчанию `180`

В общем `docker-compose.yml` API по умолчанию обращается к:

- `BACKSTORY_MODEL_BASE_URL=https://kmfj59fk-5000.euw.devtunnels.ms`
