-- Add migration script here

-- NOTE: The order matters because some tables depend on other tables
DROP TABLE IF EXISTS user_sessions;
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS todos;
DROP TABLE IF EXISTS "message";
DROP TABLE IF EXISTS member_roles;
DROP TABLE IF EXISTS channel;
DROP TABLE IF EXISTS member;
DROP TABLE IF EXISTS "role";
DROP TABLE IF EXISTS invite;
DROP TABLE IF EXISTS guild;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTGS user_relations;
DROP TYPE IF EXISTS relation_type;