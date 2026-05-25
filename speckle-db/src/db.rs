use std::path::Path;

use limbo::{Connection, Database};

use crate::DbError;
use crate::model::{
    Implementation, ImplementationAccepted, ImplementationJob, NewImplementation,
    NewImplementationAccepted, NewImplementationJob, NewSourceRange, NewSpecification,
    NewSpeckle, SourceRange, Specification, Speckle, column_integer,
};

pub const DEFAULT_PATH: &str = ".speckle/speckle.db";

const SCHEMA: &str = include_str!("schema.sql");

pub struct SpeckleDb {
    _database: Database,
    conn: Connection,
}

impl SpeckleDb {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref();
        let db = limbo::Builder::new_local(path.to_str().ok_or_else(|| {
            DbError::UnexpectedValue(format!("invalid path: {}", path.display()))
        })?)
        .build()
        .await?;
        let conn = db.connect()?;
        Ok(Self {
            _database: db,
            conn,
        })
    }

    pub async fn open_in_memory() -> Result<Self, DbError> {
        let db = limbo::Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        Ok(Self {
            _database: db,
            conn,
        })
    }

    pub async fn tx_begin(&self) -> Result<(), DbError> {
        self.conn.execute("BEGIN", ()).await?;
        Ok(())
    }

    pub async fn tx_commit(&self) -> Result<(), DbError> {
        self.conn.execute("COMMIT", ()).await?;
        Ok(())
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        for statement in split_sql_statements(SCHEMA) {
            self.conn.execute(&statement, ()).await?;
        }
        Ok(())
    }

    pub async fn insert_speckle(&self, speckle: NewSpeckle) -> Result<Speckle, DbError> {
        self.conn
            .execute(
                "INSERT INTO speckle (identifier) VALUES (?1)",
                speckle.into_params(),
            )
            .await?;
        let id = last_insert_rowid(&self.conn).await?;
        self.get_speckle_by_id(id).await
    }

    pub async fn get_speckle_by_id(&self, id: i64) -> Result<Speckle, DbError> {
        let mut rows = self
            .conn
            .query("SELECT id, identifier FROM speckle WHERE id = ?1", [id])
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        Speckle::from_row(&row)
    }

    pub async fn get_speckle_by_identifier(&self, identifier: &str) -> Result<Speckle, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, identifier FROM speckle WHERE identifier = ?1",
                [identifier],
            )
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        Speckle::from_row(&row)
    }

    pub async fn insert_source_range(
        &self,
        source_range: NewSourceRange,
    ) -> Result<SourceRange, DbError> {
        self.conn
            .execute(
                "INSERT INTO source_range (commit_hash, file_path, byte_start, byte_end) VALUES (?1, ?2, ?3, ?4)",
                source_range.into_params(),
            )
            .await?;
        let id = last_insert_rowid(&self.conn).await?;
        self.get_source_range_by_id(id).await
    }

    pub async fn get_source_range_by_id(&self, id: i64) -> Result<SourceRange, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, commit_hash, file_path, byte_start, byte_end FROM source_range WHERE id = ?1",
                [id],
            )
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        SourceRange::from_row(&row)
    }

    pub async fn insert_specification(
        &self,
        specification: NewSpecification,
    ) -> Result<Specification, DbError> {
        self.conn
            .execute(
                "INSERT INTO specification (id_speckle, id_source_range) VALUES (?1, ?2)",
                specification.into_params(),
            )
            .await?;
        let id = last_insert_rowid(&self.conn).await?;
        self.get_specification_by_id(id).await
    }

    async fn get_specification_by_id(&self, id: i64) -> Result<Specification, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, id_speckle, id_source_range FROM specification WHERE id = ?1",
                [id],
            )
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        Specification::from_row(&row)
    }

    pub async fn list_specifications_for_speckle(
        &self,
        id_speckle: i64,
    ) -> Result<Vec<Specification>, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, id_speckle, id_source_range FROM specification WHERE id_speckle = ?1 ORDER BY id",
                [id_speckle],
            )
            .await?;
        let mut specifications = Vec::new();
        while let Some(row) = rows.next().await? {
            specifications.push(Specification::from_row(&row)?);
        }
        Ok(specifications)
    }

    pub async fn insert_implementation_job(
        &self,
        job: NewImplementationJob,
    ) -> Result<ImplementationJob, DbError> {
        let (id_specification, id_external) = job.into_params();
        self.conn
            .execute(
                "INSERT INTO implementation_job (id_specification, id_external) VALUES (?1, ?2)",
                (id_specification, id_external),
            )
            .await?;
        let id = last_insert_rowid(&self.conn).await?;
        self.get_implementation_job_by_id(id).await
    }

    async fn get_implementation_job_by_id(&self, id: i64) -> Result<ImplementationJob, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, id_specification, id_external FROM implementation_job WHERE id = ?1",
                [id],
            )
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        ImplementationJob::from_row(&row)
    }

    pub async fn list_implementation_jobs_for_specification(
        &self,
        id_specification: i64,
    ) -> Result<Vec<ImplementationJob>, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, id_specification, id_external FROM implementation_job WHERE id_specification = ?1 ORDER BY id",
                [id_specification],
            )
            .await?;
        let mut jobs = Vec::new();
        while let Some(row) = rows.next().await? {
            jobs.push(ImplementationJob::from_row(&row)?);
        }
        Ok(jobs)
    }

    pub async fn get_implementation_job_by_external(
        &self,
        id_external: &str,
    ) -> Result<ImplementationJob, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, id_specification, id_external FROM implementation_job WHERE id_external = ?1",
                [id_external],
            )
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        ImplementationJob::from_row(&row)
    }

    pub async fn insert_implementation(
        &self,
        implementation: NewImplementation,
    ) -> Result<Implementation, DbError> {
        self.conn
            .execute(
                "INSERT INTO implementation (id_specification, id_source_range, source_tokens) VALUES (?1, ?2, ?3)",
                implementation.into_params(),
            )
            .await?;
        let id = last_insert_rowid(&self.conn).await?;
        self.get_implementation_by_id(id).await
    }

    async fn get_implementation_by_id(&self, id: i64) -> Result<Implementation, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, id_specification, id_source_range, source_tokens FROM implementation WHERE id = ?1",
                [id],
            )
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        Implementation::from_row(&row)
    }

    pub async fn list_implementations_for_specification(
        &self,
        id_specification: i64,
    ) -> Result<Vec<Implementation>, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, id_specification, id_source_range, source_tokens FROM implementation WHERE id_specification = ?1",
                [id_specification],
            )
            .await?;
        let mut implementations = Vec::new();
        while let Some(row) = rows.next().await? {
            implementations.push(Implementation::from_row(&row)?);
        }
        Ok(implementations)
    }

    pub async fn insert_implementation_accepted(
        &self,
        accepted: NewImplementationAccepted,
    ) -> Result<ImplementationAccepted, DbError> {
        let (id_speckle, id_implementation) = accepted.into_params();
        self.conn
            .execute(
                "INSERT INTO implementation_accepted (id_speckle, id_implementation) VALUES (?1, ?2)",
                (id_speckle, id_implementation),
            )
            .await?;
        self.get_implementation_accepted(id_speckle, id_implementation)
            .await
    }

    pub async fn get_implementation_accepted(
        &self,
        id_speckle: i64,
        id_implementation: i64,
    ) -> Result<ImplementationAccepted, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id_speckle, id_implementation FROM implementation_accepted WHERE id_speckle = ?1 AND id_implementation = ?2",
                (id_speckle, id_implementation),
            )
            .await?;
        let row = rows.next().await?.ok_or(DbError::NotFound)?;
        ImplementationAccepted::from_row(&row)
    }

    pub async fn list_implementation_accepted_for_speckle(
        &self,
        id_speckle: i64,
    ) -> Result<Vec<ImplementationAccepted>, DbError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id_speckle, id_implementation FROM implementation_accepted WHERE id_speckle = ?1",
                [id_speckle],
            )
            .await?;
        let mut accepted = Vec::new();
        while let Some(row) = rows.next().await? {
            accepted.push(ImplementationAccepted::from_row(&row)?);
        }
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

async fn last_insert_rowid(conn: &Connection) -> Result<i64, DbError> {
    let mut rows = conn.query("SELECT last_insert_rowid()", ()).await?;
    let row = rows.next().await?.ok_or_else(|| {
        DbError::UnexpectedValue("last_insert_rowid returned no rows".to_string())
    })?;
    column_integer(&row, 0, "last_insert_rowid()")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NewSourceRange;

    #[tokio::test]
    async fn test_full_insert_lookup_flow() -> Result<(), DbError> {
        let db = SpeckleDb::open_in_memory().await?;
        db.migrate().await?;

        let speckle = db
            .insert_speckle(NewSpeckle {
                identifier: "cb4cb14c-8e40-495a-b17f-6227b622f4a8".to_string(),
            })
            .await?;
        assert_eq!(
            db.get_speckle_by_id(speckle.id).await?,
            db.get_speckle_by_identifier("cb4cb14c-8e40-495a-b17f-6227b622f4a8")
                .await?
        );

        let source_range = db
            .insert_source_range(NewSourceRange {
                commit_hash: "abc123".to_string(),
                file_path: "src/lib.rs".to_string(),
                byte_start: 10,
                byte_end: 42,
            })
            .await?;
        assert_eq!(
            db.get_source_range_by_id(source_range.id).await?,
            source_range
        );

        let specification = db
            .insert_specification(NewSpecification {
                id_speckle: speckle.id,
                id_source_range: source_range.id,
            })
            .await?;
        assert_eq!(
            db.list_specifications_for_speckle(speckle.id).await?,
            vec![specification.clone()]
        );

        let job = db
            .insert_implementation_job(NewImplementationJob {
                id_specification: specification.id,
                id_external: Some("agent-run-1".to_string()),
            })
            .await?;
        assert_eq!(
            db.list_implementation_jobs_for_specification(specification.id)
                .await?,
            vec![job.clone()]
        );
        assert_eq!(
            db.get_implementation_job_by_external("agent-run-1").await?,
            job
        );

        let implementation = db
            .insert_implementation(NewImplementation {
                id_specification: specification.id,
                id_source_range: source_range.id,
                source_tokens: b"fn foo() {}".to_vec(),
            })
            .await?;
        assert_eq!(
            db.list_implementations_for_specification(specification.id)
                .await?,
            vec![implementation.clone()]
        );

        let accepted = db
            .insert_implementation_accepted(NewImplementationAccepted {
                id_speckle: speckle.id,
                id_implementation: implementation.id,
            })
            .await?;
        assert_eq!(
            db.list_implementation_accepted_for_speckle(speckle.id).await?,
            vec![accepted]
        );

        Ok(())
    }
}
