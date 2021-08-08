-- Your SQL goes here

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  username VARCHAR(64) NOT NULL,
  email VARCHAR(64) NOT NULL,
  password VARCHAR(64) NOT NULL,
  level INTEGER NOT NULL
);

CREATE TABLE contacts (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    icon BYTEA,
    creator BIGINT NOT NULL,
        
    visibility SMALLINT CHECK(visibility IN (0, 1, 2)) NOT NULL,
    -- 0 = Local
    -- 1 = Private
    -- 2 = Public

    FOREIGN KEY (creator)
        REFERENCES users(id)
);

CREATE TABLE users_contacts_join (
    user_id BIGINT NOT NULL,
    contact_id BIGINT NOT NULL,

    PRIMARY KEY (user_id, contact_id),

    FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    FOREIGN KEY (contact_id)
        REFERENCES contacts(id)
        ON DELETE CASCADE 
);

CREATE TABLE info (
    key VARCHAR(64) NOT NULL,
    value VARCHAR(512) NOT NULL,
    contact_id BIGINT NOT NULL,

    PRIMARY KEY (key, value, contact_id),

    FOREIGN KEY (contact_id)
        REFERENCES contacts(id)
        ON DELETE CASCADE
);
