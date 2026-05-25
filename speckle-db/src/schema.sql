CREATE TABLE IF NOT EXISTS speckle (
    id INTEGER PRIMARY KEY,
    identifier TEXT
);

CREATE TABLE IF NOT EXISTS specification (
    id INTEGER PRIMARY KEY,
    id_speckle INTEGER,
    version_number INTEGER,
    id_source_range INTEGER
);

CREATE TABLE IF NOT EXISTS implementation (
    id INTEGER PRIMARY KEY,
    id_specification INTEGER,
    id_source_range INTEGER
);

CREATE TABLE IF NOT EXISTS source_range (
    id INTEGER PRIMARY KEY,
    commit_hash TEXT,
    file_path TEXT,
    byte_start INTEGER,
    byte_end INTEGER
);