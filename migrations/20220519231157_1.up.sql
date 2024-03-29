-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    "id"              BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    "username"        TEXT NOT NULL UNIQUE CHECK (char_length(username) <= 64),
    "email"           TEXT NOT NULL UNIQUE CHECK (char_length(email) <= 191),
    "password"        TEXT NOT NULL,
    "profile"         TEXT,
    "created_at"      TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "description"     TEXT CHECK (char_length(description) <= 255),
    "allow_login"     BOOLEAN NOT NULL DEFAULT TRUE,
    "is_online"       BOOLEAN NOT NULL DEFAULT FALSE,
    "is_staff"        BOOLEAN NOT NULL DEFAULT FALSE,
    "is_superuser"    BOOLEAN NOT NULL DEFAULT FALSE,
    "code"            BIGINT
);

CREATE TABLE IF NOT EXISTS user_sessions (
    "session_id"      uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    "userid"          BIGINT NOT NULL,
    "last_login"      TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "device"          TEXT,
    "os"              TEXT,
    "browser"         TEXT,
    "original"        TEXT NOT NULL,
    CONSTRAINT fk_user_sessions FOREIGN KEY(userid) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS posts (
    "post_id"         BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    "created_at"      TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "updated_at"      TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "title"           TEXT NOT NULL CHECK (char_length(title) <= 255),
    "content"         TEXT,
    "published"       BOOLEAN DEFAULT FALSE,
    "authorid"        BIGINT NOT NULL,
    CONSTRAINT fk_user_posts FOREIGN KEY(authorid) REFERENCES users(id) ON DELETE CASCADE
);


--- WARNING: Below contains a bunch of epic sql code
---          which prefectly replicates discord's models (or atleast as close as it gets)

CREATE TABLE IF NOT EXISTS "guild" (
    "id"          uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    "name"        varchar(50) NOT NULL,
    "description" text NULL,
    "icon"        varchar(100) NULL,
    "created_at"  TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "creator_id"  BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

-- channel_type (s):
-- 0: text channel
-- 1: dm channel (text)
-- 2: category channel

CREATE TABLE IF NOT EXISTS "channel" (
    "id"            uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    "name"          varchar(50) NOT NULL,
    "description"   text NULL,
    "position"      bigint NOT NULL CHECK ("position" >= 0),
    "created_at"    TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "channel_type"  SMALLINT NOT NULL,
    "guild_id"      uuid REFERENCES guild(id) ON DELETE CASCADE,
-- Either guild_id presents or both user fields presents
    "user1"         BIGINT REFERENCES users(id),
    "user2"         BIGINT REFERENCES users(id),
    UNIQUE (position, guild_id),
    UNIQUE (name, guild_id),
    -- UNIQUE (user1, user2),
    CONSTRAINT DIFFERENT_USERS_CHECK
    CHECK (user1 <> user2),
    CONSTRAINT DM_CHANNEL_CHECK
    CHECK ((guild_id IS NOT NULL AND user1 IS NULL AND user2 IS NULL) OR (guild_id IS NULL AND user1 IS NOT NULL AND user2 IS NOT NULL) )
);

CREATE UNIQUE INDEX ON channel (least(user1, user2), greatest(user1, user2));


CREATE TABLE IF NOT EXISTS "member" (
    "id"        uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    "nick_name" varchar(25) NULL,
    "joined_at" TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "guild_id"  uuid NOT NULL REFERENCES guild(id) ON DELETE CASCADE,
    "user_id"   BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS "role" (
    "id"              uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    "name"            varchar(25) NOT NULL,
    "colour"          varchar(15) NOT NULL,
    "position"        integer NOT NULL CHECK ("position" >= 0),
    "created_at"      TIMESTAMP DEFAULT current_timestamp NOT NULL ,
    "guild_id"        uuid NOT NULL REFERENCES guild(id) ON DELETE CASCADE,
    --- Role permission fields
    --- Allow to read and send messages by default
    "permissions" integer
);

CREATE TABLE IF NOT EXISTS "message" (
    "id"         uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    "content"    text NOT NULL,
    "created_at" TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "edited_at"  TIMESTAMP DEFAULT current_timestamp NOT NULL,  --- because you said null vals can cause issues
    "author_id"  bigint NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    "channel_id" uuid NOT NULL REFERENCES channel(id) ON DELETE CASCADE
);

--- Roles and member have a many to many relationship
CREATE TABLE IF NOT EXISTS "member_roles" (
    "id"        bigserial NOT NULL PRIMARY KEY,
    "member_id" uuid NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    "role_id"   uuid NOT NULL REFERENCES "role"(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS "invite" (
    "code"       varchar(8) NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP DEFAULT current_timestamp NOT NULL,
    "guild_id"   uuid NOT NULL UNIQUE REFERENCES guild(id) ON DELETE CASCADE
);

--create types
-- DO $$
-- BEGIN
--     IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'relation_type') THEN

-- Note: CREATE TYPE IF NOT EXISTS syntax only works with cockroach labs for some reason
        -- CREATE TYPE IF NOT EXISTS relation_type AS ENUM ('outgoing', 'ongoing', 'friend', 'block');
        CREATE TYPE relation_type AS ENUM ('outgoing', 'ongoing', 'friend', 'block');
--     END IF;
--     --more types here...
-- END$$;

CREATE TABLE IF NOT EXISTS "user_relations" (
    "id"              uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    "user1"           BIGINT NOT NULL REFERENCES users(id),
    "user2"           BIGINT NOT NULL REFERENCES users(id),
    "relationship"    relation_type
);