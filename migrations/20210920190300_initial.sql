-- Add migration script here
CREATE TABLE valorant_accounts (
	account_id INTEGER PRIMARY KEY,
	discord_id TEXT NOT NULL,
	username TEXT NOT NULL,
	password TEXT NOT NULL
);