CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    userid TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    nickname TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX users_userid_idx ON users (userid);

CREATE TABLE jobs (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    factors INTEGER[6]
);

CREATE TABLE states (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE characters (
    id SERIAL PRIMARY KEY,
    firstname TEXT NOT NULL,
    surname TEXT UNIQUE NOT NULL,
    matherid INTEGER REFERENCES characters (id),
    fatherid INTEGER REFERENCES characters (id),
    partnerid INTEGER REFERENCES characters (id),
    ownerid INTEGER REFERENCES users (id),
    seed FLOAT[] NOT NULL,
    url TEXT NOT NULL,
    jobid INTEGER REFERENCES jobs (id),
    height FLOAT NOT NULL DEFAULT 160.0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    stats INTEGER[6] NOT NULL,
    stateid INTEGER NOT NULL DEFAULT 0 REFERENCES states (id)
);
CREATE INDEX characters_ownerid_idx ON characters (ownerid);
CREATE INDEX characters_materid_idx ON characters (matherid);
CREATE INDEX characters_fatherid_idx ON characters (fatherid);



CREATE TABLE actions (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    duration INTERVAL
);
INSERT INTO actions (id, name, description, duration) VALUES (1, 'random_gen_character', 'random character generation action', '01:00:00');

CREATE TABLE users_actions (
    id SERIAL PRIMARY KEY,
    userid INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
    actionid INTEGER NOT NULL REFERENCES actions (id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX users_actions_userid_idx ON users_actions (id, userid, actionid);


CREATE TABLE traits (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    duration INTERVAL
);

CREATE TABLE characters_traits (
    id SERIAL PRIMARY KEY,
    characterid INTEGER REFERENCES characters (id) ON DELETE CASCADE ON UPDATE CASCADE,
    traitid INTEGER REFERENCES traits (id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX characters_traits_charid ON characters_traits (characterid);
