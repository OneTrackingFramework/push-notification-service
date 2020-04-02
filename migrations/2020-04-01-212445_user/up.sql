-- Your SQL goes here
CREATE TABLE IF NOT EXISTS pdevicetype (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) UNIQUE NOT NULL
);

INSERT INTO pdevicetype(name)
VALUES('IOS');
INSERT INTO pdevicetype(name)
VALUES('FIREBASE');

CREATE TABLE IF NOT EXISTS puser (
            id SERIAL PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            device_type_id INTEGER NOT NULL REFERENCES pdevicetype(id) ON DELETE CASCADE,
            token VARCHAR(255) UNIQUE NOT NULL
);