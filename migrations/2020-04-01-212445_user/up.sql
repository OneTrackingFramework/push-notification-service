-- Your SQL goes here
CREATE TABLE IF NOT EXISTS puser (
            id SERIAL PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            token VARCHAR(255) UNIQUE NOT NULL
);