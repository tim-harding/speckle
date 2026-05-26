-- Each `#[speckle]` attribute is a row.
CREATE TABLE IF NOT EXISTS speckle (
    -- The identifier within the database.
    -- However, the true identity is the text identifier below.
    id INTEGER PRIMARY KEY,
    -- The stable text identifier of the attribute.
    -- For example, `#[speckle("deadbeef")]` has the identifier `deadbeef`.
    -- The text identifier is required. 
    -- It may be populated automatically by cargo-speckle,
    -- but the row will not be inserted without a stable identity.
    identifier TEXT NOT NULL UNIQUE
);

-- To look up a speckle by its stable identifier.
CREATE INDEX IF NOT EXISTS idx_speckle_identifier ON speckle (identifier);

-- Points to a source location where the author wrote the specification.
-- There may be multiple entries tracking the evolution of the specification.
CREATE TABLE IF NOT EXISTS specification (
    -- In this case, the ID also implies the version history order.
    id INTEGER PRIMARY KEY,
    -- The speckle this revision belongs to.
    id_speckle INTEGER NOT NULL,
    -- The source location of the annotated item.
    id_source_range INTEGER NOT NULL
);

-- To look up all specifications for a given speckle.
CREATE INDEX IF NOT EXISTS idx_id_speckle ON specification (id_speckle);

-- Points to a job that is implementing a specification.
-- There may be multiple jobs if, for example, there are 
-- failures, cancellations, retries, parallel attempts, etcetera.
-- It is up to an external tool to orchestrate the agentic work;
-- Speckle itself is unopionated in this regard.
CREATE TABLE IF NOT EXISTS implementation_job (
    id INTEGER PRIMARY KEY,
    -- The specification being implemented.
    id_specification INTEGER NOT NULL,
    -- The identifier the external tool uses to track the job.
    -- Optional for flexibility; the external tool may elect to ignore it.
    id_external TEXT
);

-- To look up all jobs for a given specification.
CREATE INDEX IF NOT EXISTS idx_id_specification ON implementation_job (id_specification);
-- To look up a job by its external identifier.
CREATE INDEX IF NOT EXISTS idx_id_external ON implementation_job (id_external);

-- A pointer to a source location where the agent implemented a specification revision.
CREATE TABLE IF NOT EXISTS implementation (
    id INTEGER PRIMARY KEY,
    -- The specification that was implemented.
    id_specification INTEGER NOT NULL,
    -- The job that implemented the specification.
    -- Optional in case, for example, it was implemented by hand.
    id_implementation_job INTEGER,
    -- The source location of the implemented item.
    id_source_range INTEGER NOT NULL,
    -- Text to substitute for the content span of the annotated item.
    -- For example, the content of a function is its body.
    -- This is inlined in zero-copy form 
    -- rather than derived from the repository state
    -- as a caching optimization for the proc macro.
    source_tokens BLOB NOT NULL
);

-- To look up all implementations for a given specification.
CREATE INDEX IF NOT EXISTS idx_implementation_id_specification ON implementation (id_specification);
-- To look up the implementation produced by a given job.
CREATE INDEX IF NOT EXISTS idx_implementation_id_implementation_job ON implementation (id_implementation_job);

-- A record of the accepted implementation of a specification
-- that the macro will substitute for the annotated item.
CREATE TABLE IF NOT EXISTS implementation_accepted (
    -- The speckle that accepted the implementation.
    id_speckle INTEGER,
    -- The implementation that was accepted.
    id_implementation INTEGER,
    PRIMARY KEY (id_speckle, id_implementation)
) WITHOUT ROWID;

-- A record of a source location in the repository history.
CREATE TABLE IF NOT EXISTS source_range (
    id INTEGER PRIMARY KEY,
    commit_hash TEXT NOT NULL,
    file_path TEXT NOT NULL,
    byte_start INTEGER NOT NULL,
    byte_end INTEGER NOT NULL
);
