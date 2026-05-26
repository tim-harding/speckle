use std::path::Path;

use rusqlite::{Connection, Transaction};

use crate::DbError;
use crate::model::{
    Implementation, ImplementationAccepted, ImplementationJob, NewImplementation,
    NewImplementationAccepted, NewImplementationJob, NewSourceRange, NewSpecification, NewSpeckle,
    SourceRange, Specification, Speckle,
};

pub const DEFAULT_PATH: &str = ".speckle/db.sqlite3";

const SCHEMA: &str = include_str!("schema.sql");

pub struct SpeckleDb {
    conn: Connection,
}

pub struct SpeckleDbSession<'db> {
    tx: Transaction<'db>,
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

    pub fn transaction(&mut self) -> Result<SpeckleDbSession<'_>, DbError> {
        Ok(SpeckleDbSession {
            tx: self.conn.transaction()?,
        })
    }

    pub fn migrate(&self) -> Result<(), DbError> {
        self.conn.execute_batch(SCHEMA)?;
        Ok(())
    }

    pub fn get_speckle_by_id(&self, id: i64) -> Result<Speckle, DbError> {
        self.conn
            .query_row(
                "SELECT id, identifier FROM speckle WHERE id = ?1",
                [id],
                |row| Speckle::from_row(row),
            )
            .map_err(map_not_found)
    }

    pub fn get_speckle_by_identifier(&self, identifier: &str) -> Result<Speckle, DbError> {
        self.conn
            .query_row(
                "SELECT id, identifier FROM speckle WHERE identifier = ?1",
                [identifier],
                |row| Speckle::from_row(row),
            )
            .map_err(map_not_found)
    }

    pub fn get_source_range_by_id(&self, id: i64) -> Result<SourceRange, DbError> {
        self.conn
            .query_row(
                "SELECT id, commit_hash, file_path, byte_start, byte_end FROM source_range WHERE id = ?1",
                [id],
                |row| SourceRange::from_row(row),
            )
            .map_err(map_not_found)
    }

    pub fn list_specifications_for_speckle(
        &self,
        id_speckle: i64,
    ) -> Result<Vec<Specification>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, id_speckle, id_source_range, source_pod FROM specification WHERE id_speckle = ?1 ORDER BY id",
        )?;
        let specifications = stmt
            .query_map([id_speckle], Specification::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(specifications)
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
            .map_err(map_not_found)
    }

    pub fn list_implementations_for_specification(
        &self,
        id_specification: i64,
    ) -> Result<Vec<Implementation>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, id_specification, id_implementation_job, id_source_range, source_pod FROM implementation WHERE id_specification = ?1",
        )?;
        let implementations = stmt
            .query_map([id_specification], Implementation::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(implementations)
    }

    pub fn list_implementations_for_implementation_job(
        &self,
        id_implementation_job: i64,
    ) -> Result<Vec<Implementation>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, id_specification, id_implementation_job, id_source_range, source_pod FROM implementation WHERE id_implementation_job = ?1",
        )?;
        let implementations = stmt
            .query_map([id_implementation_job], Implementation::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(implementations)
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
            .map_err(map_not_found)
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

impl SpeckleDbSession<'_> {
    pub fn commit(self) -> Result<(), DbError> {
        self.tx.commit()?;
        Ok(())
    }

    pub fn insert_speckle(&mut self, speckle: NewSpeckle) -> Result<Speckle, DbError> {
        self.tx.execute(
            "INSERT INTO speckle (identifier) VALUES (?1)",
            speckle.into_params(),
        )?;
        let id = self.tx.last_insert_rowid();
        self.get_speckle_by_id(id)
    }

    pub fn insert_source_range(
        &mut self,
        source_range: NewSourceRange,
    ) -> Result<SourceRange, DbError> {
        self.tx.execute(
            "INSERT INTO source_range (commit_hash, file_path, byte_start, byte_end) VALUES (?1, ?2, ?3, ?4)",
            source_range.into_params(),
        )?;
        let id = self.tx.last_insert_rowid();
        self.get_source_range_by_id(id)
    }

    pub fn insert_specification(
        &mut self,
        specification: NewSpecification,
    ) -> Result<Specification, DbError> {
        self.tx.execute(
            "INSERT INTO specification (id_speckle, id_source_range, source_pod) VALUES (?1, ?2, ?3)",
            specification.into_params(),
        )?;
        let id = self.tx.last_insert_rowid();
        self.get_specification_by_id(id)
    }

    pub fn insert_implementation_job(
        &mut self,
        job: NewImplementationJob,
    ) -> Result<ImplementationJob, DbError> {
        let (id_specification, id_external) = job.into_params();
        self.tx.execute(
            "INSERT INTO implementation_job (id_specification, id_external) VALUES (?1, ?2)",
            (id_specification, id_external),
        )?;
        let id = self.tx.last_insert_rowid();
        self.get_implementation_job_by_id(id)
    }

    pub fn insert_implementation(
        &mut self,
        implementation: NewImplementation,
    ) -> Result<Implementation, DbError> {
        self.tx.execute(
            "INSERT INTO implementation (id_specification, id_implementation_job, id_source_range, source_pod) VALUES (?1, ?2, ?3, ?4)",
            implementation.into_params(),
        )?;
        let id = self.tx.last_insert_rowid();
        self.get_implementation_by_id(id)
    }

    pub fn insert_implementation_accepted(
        &mut self,
        accepted: NewImplementationAccepted,
    ) -> Result<ImplementationAccepted, DbError> {
        let (id_speckle, id_implementation) = accepted.into_params();
        self.tx.execute(
            "INSERT INTO implementation_accepted (id_speckle, id_implementation) VALUES (?1, ?2)",
            (id_speckle, id_implementation),
        )?;
        self.get_implementation_accepted(id_speckle, id_implementation)
    }

    fn get_speckle_by_id(&self, id: i64) -> Result<Speckle, DbError> {
        self.tx
            .query_row(
                "SELECT id, identifier FROM speckle WHERE id = ?1",
                [id],
                |row| Speckle::from_row(row),
            )
            .map_err(map_not_found)
    }

    fn get_source_range_by_id(&self, id: i64) -> Result<SourceRange, DbError> {
        self.tx
            .query_row(
                "SELECT id, commit_hash, file_path, byte_start, byte_end FROM source_range WHERE id = ?1",
                [id],
                |row| SourceRange::from_row(row),
            )
            .map_err(map_not_found)
    }

    fn get_specification_by_id(&self, id: i64) -> Result<Specification, DbError> {
        self.tx
            .query_row(
                "SELECT id, id_speckle, id_source_range, source_pod FROM specification WHERE id = ?1",
                [id],
                |row| Specification::from_row(row),
            )
            .map_err(map_not_found)
    }

    fn get_implementation_job_by_id(&self, id: i64) -> Result<ImplementationJob, DbError> {
        self.tx
            .query_row(
                "SELECT id, id_specification, id_external FROM implementation_job WHERE id = ?1",
                [id],
                |row| ImplementationJob::from_row(row),
            )
            .map_err(map_not_found)
    }

    fn get_implementation_by_id(&self, id: i64) -> Result<Implementation, DbError> {
        self.tx
            .query_row(
                "SELECT id, id_specification, id_implementation_job, id_source_range, source_pod FROM implementation WHERE id = ?1",
                [id],
                |row| Implementation::from_row(row),
            )
            .map_err(map_not_found)
    }

    fn get_implementation_accepted(
        &self,
        id_speckle: i64,
        id_implementation: i64,
    ) -> Result<ImplementationAccepted, DbError> {
        self.tx
            .query_row(
                "SELECT id_speckle, id_implementation FROM implementation_accepted WHERE id_speckle = ?1 AND id_implementation = ?2",
                (id_speckle, id_implementation),
                |row| ImplementationAccepted::from_row(row),
            )
            .map_err(map_not_found)
    }
}

fn map_not_found(error: rusqlite::Error) -> DbError {
    match error {
        rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
        other => other.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NewSourceRange;
    use speckle_syntax::{ItemKind, StoredItem, StoredSpanContent};

    #[test]
    fn test_full_insert_lookup_flow() -> Result<(), DbError> {
        let mut db = SpeckleDb::open_in_memory()?;
        db.migrate()?;

        let mut tx = db.transaction()?;
        let speckle = tx.insert_speckle(NewSpeckle {
            identifier: "cb4cb14c-8e40-495a-b17f-6227b622f4a8".to_string(),
        })?;

        let source_range = tx.insert_source_range(NewSourceRange {
            commit_hash: "abc123".to_string(),
            file_path: "src/lib.rs".to_string(),
            byte_start: 10,
            byte_end: 42,
        })?;

        let specification = tx.insert_specification(NewSpecification::from_stored_item(
            speckle.id,
            source_range.id,
            &StoredItem {
                kind: ItemKind::Fn,
                speckle_arguments: Vec::new(),
                content: "{}".to_string(),
            },
        )?)?;

        let job = tx.insert_implementation_job(NewImplementationJob {
            id_specification: specification.id,
            id_external: Some("agent-run-1".to_string()),
        })?;

        let implementation =
            tx.insert_implementation(NewImplementation::from_stored_span_content(
                specification.id,
                Some(job.id),
                source_range.id,
                &StoredSpanContent {
                    content: "fn foo() {}".to_string(),
                },
            )?)?;

        let accepted = tx.insert_implementation_accepted(NewImplementationAccepted {
            id_speckle: speckle.id,
            id_implementation: implementation.id,
        })?;
        tx.commit()?;

        assert_eq!(
            db.get_speckle_by_id(speckle.id)?,
            db.get_speckle_by_identifier("cb4cb14c-8e40-495a-b17f-6227b622f4a8")?
        );
        assert_eq!(db.get_source_range_by_id(source_range.id)?, source_range);
        assert_eq!(
            db.list_specifications_for_speckle(speckle.id)?,
            vec![specification.clone()]
        );
        assert_eq!(
            db.list_implementation_jobs_for_specification(specification.id)?,
            vec![job.clone()]
        );
        assert_eq!(db.get_implementation_job_by_external("agent-run-1")?, job);
        assert_eq!(
            db.list_implementations_for_specification(specification.id)?,
            vec![implementation.clone()]
        );
        assert_eq!(
            db.list_implementations_for_implementation_job(job.id)?,
            vec![implementation.clone()]
        );
        assert_eq!(
            db.list_implementation_accepted_for_speckle(speckle.id)?,
            vec![accepted]
        );

        Ok(())
    }
}
