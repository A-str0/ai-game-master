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

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'ContextObjectType') THEN
        CREATE TYPE "ContextObjectType" AS ENUM ('Npc', 'Place', 'Item', 'Event', 'Note');
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

CREATE TABLE IF NOT EXISTS context_objects (
    id UUID PRIMARY KEY,
    object_type "ContextObjectType" NOT NULL,
    title TEXT NOT NULL,
    short_desc TEXT NOT NULL,
    long_desc TEXT NULL,
    attributes JSON NOT NULL,
    place_id UUID NULL REFERENCES context_objects(id) ON DELETE SET NULL,
    importance_score REAL NOT NULL,
    created_by TEXT NOT NULL,
    seed BIGINT NOT NULL,
    created_ts TIMESTAMPTZ NOT NULL,
    updated_ts TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_context_objects_place_id
    ON context_objects (place_id);
