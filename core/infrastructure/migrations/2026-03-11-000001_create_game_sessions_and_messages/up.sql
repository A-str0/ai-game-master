CREATE TYPE "SessionMode" AS ENUM ('Solo', 'Multi');
CREATE TYPE "MessageRole" AS ENUM ('Player', 'Gm', 'System');

CREATE TABLE game_sessions (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    retrivial_k SMALLINT NOT NULL CHECK (retrivial_k > 0),
    memory_budget INTEGER NOT NULL CHECK (memory_budget > 0),
    session_mode "SessionMode" NOT NULL,
    created_ts TIMESTAMPTZ NOT NULL,
    last_activity_ts TIMESTAMPTZ NULL
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    role "MessageRole" NOT NULL,
    text TEXT NOT NULL CHECK (btrim(text) <> ''),
    ts TIMESTAMPTZ NOT NULL
);

CREATE INDEX messages_session_id_ts_idx
    ON messages (session_id, ts DESC, id DESC);
