-- Your SQL goes here
CREATE TABLE IF NOT EXISTS pdevicetype (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) UNIQUE NOT NULL
);

INSERT INTO pdevicetype(name)
VALUES('IOS');
INSERT INTO pdevicetype(name)
VALUES('FIREBASE');

CREATE TABLE IF NOT EXISTS pdevice (
            id SERIAL PRIMARY KEY,
            devicetypeid INTEGER NOT NULL REFERENCES pdevicetype(id),
            token VARCHAR(255) UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS puser (
            id SERIAL PRIMARY KEY,
            userid VARCHAR(255) NOT NULL,
            deviceid INTEGER NOT NULL REFERENCES pdevice(id)
);