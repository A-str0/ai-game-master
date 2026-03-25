DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'SessionMode') THEN
        CREATE TYPE "SessionMode" AS ENUM ('Solo', 'Multi');
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'MessageRole') THEN
        CREATE TYPE "MessageRole" AS ENUM ('Player', 'Gm', 'System');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS game_sessions (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    retrivial_k SMALLINT NOT NULL,
    memory_budget INTEGER NOT NULL,
    session_mode "SessionMode" NOT NULL,
    created_ts TIMESTAMPTZ NOT NULL,
    last_activity_ts TIMESTAMPTZ NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    role "MessageRole" NOT NULL,
    text TEXT NOT NULL,
    ts TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_session_ts_id
    ON messages (session_id, ts DESC, id DESC);
