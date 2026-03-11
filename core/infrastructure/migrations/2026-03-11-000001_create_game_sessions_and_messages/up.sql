DO $$
BEGIN
    CREATE TYPE "SessionMode" AS ENUM ('Solo', 'Multi');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$
BEGIN
    CREATE TYPE "MessageRole" AS ENUM ('Player', 'Gm', 'System');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS game_sessions (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    retrivial_k SMALLINT NOT NULL CHECK (retrivial_k > 0),
    memory_budget INTEGER NOT NULL CHECK (memory_budget > 0),
    session_mode "SessionMode" NOT NULL,
    created_ts TIMESTAMPTZ NOT NULL,
    last_activity_ts TIMESTAMPTZ NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    role "MessageRole" NOT NULL,
    text TEXT NOT NULL CHECK (btrim(text) <> ''),
    ts TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS messages_session_id_ts_idx
    ON messages (session_id, ts DESC, id DESC);
