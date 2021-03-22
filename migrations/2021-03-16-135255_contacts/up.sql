-- Your SQL goes here
CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  username VARCHAR(64) NOT NULL,
  email VARCHAR(64) NOT NULL,
  password VARCHAR(64) NOT NULL
);

CREATE TABLE personas (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    private BOOLEAN NOT NULL DEFAULT FALSE,
    user_id BIGINT NOT NULL,

    UNIQUE (user_id, name),

    FOREIGN KEY (user_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE
);

CREATE TABLE contacts (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    birthday DATE,
    icon BYTEA,
    persona BIGINT NULL UNIQUE,

    FOREIGN KEY (persona)
        REFERENCES personas(id)
);

CREATE TABLE users_contacts_join (
    user_id BIGINT NOT NULL UNIQUE,
    contact_id BIGINT NOT NULL UNIQUE,

    PRIMARY KEY (user_id, contact_id),

    FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    FOREIGN KEY (contact_id)
        REFERENCES contacts(id)
        ON DELETE CASCADE 
);

CREATE TABLE phones (
    id BIGSERIAL PRIMARY KEY,
    phone VARCHAR(64) NOT NULL,
    contact_id BIGINT NOT NULL UNIQUE,

    FOREIGN KEY (contact_id) 
        REFERENCES contacts(id) 
        ON DELETE CASCADE
);

CREATE TABLE emails (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(64) NOT NULL,
    contact_id BIGINT NOT NULL UNIQUE,

    FOREIGN KEY (contact_id) 
        REFERENCES contacts(id) 
        ON DELETE CASCADE
);

CREATE TABLE notes (
    id BIGSERIAL PRIMARY KEY,
    note TEXT NOT NULL,
    contact_id BIGINT NOT NULL UNIQUE,

    FOREIGN KEY (contact_id) 
        REFERENCES contacts(id) 
        ON DELETE CASCADE
);

CREATE TABLE social_media (
    id BIGSERIAL PRIMARY KEY,
    link VARCHAR(64) NOT NULL,
    type VARCHAR(64) NOT NULL,
    contact_id BIGINT NOT NULL UNIQUE,

    FOREIGN KEY (contact_id) 
        REFERENCES contacts(id) 
        ON DELETE CASCADE
);

CREATE TABLE websites (
    id BIGSERIAL PRIMARY KEY,
    link VARCHAR(64) NOT NULL,
    contact_id BIGINT NOT NULL UNIQUE,

    FOREIGN KEY (contact_id) 
        REFERENCES contacts(id) 
        ON DELETE CASCADE
);

CREATE TABLE addresses (
    id BIGSERIAL PRIMARY KEY,
    street VARCHAR(64),
    locality VARCHAR(64),
    postal_code INT,
    country VARCHAR(64),
    contact_id BIGINT NOT NULL UNIQUE,

    FOREIGN KEY (contact_id) 
        REFERENCES contacts(id) 
        ON DELETE CASCADE
);

CREATE TABLE anniversaries (
    id BIGSERIAL PRIMARY KEY,
    date DATE NOT NULL,
    contact_id BIGINT NOT NULL UNIQUE,

    FOREIGN KEY (contact_id) 
        REFERENCES contacts(id) 
        ON DELETE CASCADE
);