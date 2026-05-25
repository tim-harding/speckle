use std::path::Path;

use rusqlite::Connection;

use crate::DbError;
use crate::model::{
    Implementation, ImplementationAccepted, ImplementationJob, NewImplementation,
    NewImplementationAccepted, NewImplementationJob, NewSourceRange, NewSpecification,
    NewSpeckle, SourceRange, Specification, Speckle,
};

pub const DEFAULT_PATH: &str = ".speckle/speckle.db";

const SCHEMA: &str = include_str!("schema.sql");

pub struct SpeckleDb {
    conn: Connection,
}

impl SpeckleDb {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    pub fn tx_begin(&self) -> Result<(), DbError> {
        self.conn.execute("BEGIN", ())?;
        Ok(())
    }

    pub fn tx_commit(&self) -> Result<(), DbError> {
        self.conn.execute("COMMIT", ())?;
        Ok(())
    }

    pub fn migrate(&self) -> Result<(), DbError> {
        for statement in split_sql_statements(SCHEMA) {
            self.conn.execute(&statement, ())?;
        }
        Ok(())
    }

    pub fn insert_speckle(&self, speckle: NewSpeckle) -> Result<Speckle, DbError> {
        self.conn.execute(
            "INSERT INTO speckle (identifier) VALUES (?1)",
            speckle.into_params(),
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_speckle_by_id(id)
    }

    pub fn get_speckle_by_id(&self, id: i64) -> Result<Speckle, DbError> {
        self.conn
            .query_row(
                "SELECT id, identifier FROM speckle WHERE id = ?1",
                [id],
                |row| Speckle::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn get_speckle_by_identifier(&self, identifier: &str) -> Result<Speckle, DbError> {
        self.conn
            .query_row(
                "SELECT id, identifier FROM speckle WHERE identifier = ?1",
                [identifier],
                |row| Speckle::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn insert_source_range(
        &self,
        source_range: NewSourceRange,
    ) -> Result<SourceRange, DbError> {
        self.conn.execute(
            "INSERT INTO source_range (commit_hash, file_path, byte_start, byte_end) VALUES (?1, ?2, ?3, ?4)",
            source_range.into_params(),
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_source_range_by_id(id)
    }

    pub fn get_source_range_by_id(&self, id: i64) -> Result<SourceRange, DbError> {
        self.conn
            .query_row(
                "SELECT id, commit_hash, file_path, byte_start, byte_end FROM source_range WHERE id = ?1",
                [id],
                |row| SourceRange::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn insert_specification(
        &self,
        specification: NewSpecification,
    ) -> Result<Specification, DbError> {
        self.conn.execute(
            "INSERT INTO specification (id_speckle, id_source_range) VALUES (?1, ?2)",
            specification.into_params(),
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_specification_by_id(id)
    }

    fn get_specification_by_id(&self, id: i64) -> Result<Specification, DbError> {
        self.conn
            .query_row(
                "SELECT id, id_speckle, id_source_range FROM specification WHERE id = ?1",
                [id],
                |row| Specification::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn list_specifications_for_speckle(
        &self,
        id_speckle: i64,
    ) -> Result<Vec<Specification>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, id_speckle, id_source_range FROM specification WHERE id_speckle = ?1 ORDER BY id",
        )?;
        let specifications = stmt
            .query_map([id_speckle], Specification::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(specifications)
    }

    pub fn insert_implementation_job(
        &self,
        job: NewImplementationJob,
    ) -> Result<ImplementationJob, DbError> {
        let (id_specification, id_external) = job.into_params();
        self.conn.execute(
            "INSERT INTO implementation_job (id_specification, id_external) VALUES (?1, ?2)",
            (id_specification, id_external),
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_implementation_job_by_id(id)
    }

    fn get_implementation_job_by_id(&self, id: i64) -> Result<ImplementationJob, DbError> {
        self.conn
            .query_row(
                "SELECT id, id_specification, id_external FROM implementation_job WHERE id = ?1",
                [id],
                |row| ImplementationJob::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn list_implementation_jobs_for_specification(
        &self,
        id_specification: i64,
    ) -> Result<Vec<ImplementationJob>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, id_specification, id_external FROM implementation_job WHERE id_specification = ?1 ORDER BY id",
        )?;
        let jobs = stmt
            .query_map([id_specification], ImplementationJob::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(jobs)
    }

    pub fn get_implementation_job_by_external(
        &self,
        id_external: &str,
    ) -> Result<ImplementationJob, DbError> {
        self.conn
            .query_row(
                "SELECT id, id_specification, id_external FROM implementation_job WHERE id_external = ?1",
                [id_external],
                |row| ImplementationJob::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn insert_implementation(
        &self,
        implementation: NewImplementation,
    ) -> Result<Implementation, DbError> {
        self.conn.execute(
            "INSERT INTO implementation (id_specification, id_source_range, source_tokens) VALUES (?1, ?2, ?3)",
            implementation.into_params(),
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_implementation_by_id(id)
    }

    fn get_implementation_by_id(&self, id: i64) -> Result<Implementation, DbError> {
        self.conn
            .query_row(
                "SELECT id, id_specification, id_source_range, source_tokens FROM implementation WHERE id = ?1",
                [id],
                |row| Implementation::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn list_implementations_for_specification(
        &self,
        id_specification: i64,
    ) -> Result<Vec<Implementation>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, id_specification, id_source_range, source_tokens FROM implementation WHERE id_specification = ?1",
        )?;
        let implementations = stmt
            .query_map([id_specification], Implementation::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(implementations)
    }

    pub fn insert_implementation_accepted(
        &self,
        accepted: NewImplementationAccepted,
    ) -> Result<ImplementationAccepted, DbError> {
        let (id_speckle, id_implementation) = accepted.into_params();
        self.conn.execute(
            "INSERT INTO implementation_accepted (id_speckle, id_implementation) VALUES (?1, ?2)",
            (id_speckle, id_implementation),
        )?;
        self.get_implementation_accepted(id_speckle, id_implementation)
    }

    pub fn get_implementation_accepted(
        &self,
        id_speckle: i64,
        id_implementation: i64,
    ) -> Result<ImplementationAccepted, DbError> {
        self.conn
            .query_row(
                "SELECT id_speckle, id_implementation FROM implementation_accepted WHERE id_speckle = ?1 AND id_implementation = ?2",
                (id_speckle, id_implementation),
                |row| ImplementationAccepted::from_row(row),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
                other => other.into(),
            })
    }

    pub fn list_implementation_accepted_for_speckle(
        &self,
        id_speckle: i64,
    ) -> Result<Vec<ImplementationAccepted>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id_speckle, id_implementation FROM implementation_accepted WHERE id_speckle = ?1",
        )?;
        let accepted = stmt
            .query_map([id_speckle], ImplementationAccepted::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(accepted)
    }
}

fn split_sql_statements(schema: &str) -> Vec<String> {
    let stripped = schema
        .lines()
        .map(|line| {
            if let Some(comment_start) = line.find("--") {
                &line[..comment_start]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    stripped
        .split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NewSourceRange;

    #[test]
    fn test_full_insert_lookup_flow() -> Result<(), DbError> {
        let db = SpeckleDb::open_in_memory()?;
        db.migrate()?;

        let speckle = db.insert_speckle(NewSpeckle {
            identifier: "cb4cb14c-8e40-495a-b17f-6227b622f4a8".to_string(),
        })?;
        assert_eq!(
            db.get_speckle_by_id(speckle.id)?,
            db.get_speckle_by_identifier("cb4cb14c-8e40-495a-b17f-6227b622f4a8")?
        );

        let source_range = db.insert_source_range(NewSourceRange {
            commit_hash: "abc123".to_string(),
            file_path: "src/lib.rs".to_string(),
            byte_start: 10,
            byte_end: 42,
        })?;
        assert_eq!(
            db.get_source_range_by_id(source_range.id)?,
            source_range
        );

        let specification = db.insert_specification(NewSpecification {
            id_speckle: speckle.id,
            id_source_range: source_range.id,
        })?;
        assert_eq!(
            db.list_specifications_for_speckle(speckle.id)?,
            vec![specification.clone()]
        );

        let job = db.insert_implementation_job(NewImplementationJob {
            id_specification: specification.id,
            id_external: Some("agent-run-1".to_string()),
        })?;
        assert_eq!(
            db.list_implementation_jobs_for_specification(specification.id)?,
            vec![job.clone()]
        );
        assert_eq!(
            db.get_implementation_job_by_external("agent-run-1")?,
            job
        );

        let implementation = db.insert_implementation(NewImplementation {
            id_specification: specification.id,
            id_source_range: source_range.id,
            source_tokens: b"fn foo() {}".to_vec(),
        })?;
        assert_eq!(
            db.list_implementations_for_specification(specification.id)?,
            vec![implementation.clone()]
        );

        let accepted = db.insert_implementation_accepted(NewImplementationAccepted {
            id_speckle: speckle.id,
            id_implementation: implementation.id,
        })?;
        assert_eq!(
            db.list_implementation_accepted_for_speckle(speckle.id)?,
            vec![accepted]
        );

        Ok(())
    }
}
