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
