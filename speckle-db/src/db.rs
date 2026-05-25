use std::path::Path;

use limbo::{Connection, Database};

use crate::DbError;
use crate::model::{
    Implementation, NewImplementation, NewSourceRange, NewSpecification, NewSpeckle, SourceRange,
    Specification, Speckle, column_integer,
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
        for statement in SCHEMA.split(';') {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }
            self.conn.execute(statement, ()).await?;
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
                "INSERT INTO specification (id_speckle, version_number, id_source_range) VALUES (?1, ?2, ?3)",
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
                "SELECT id, id_speckle, version_number, id_source_range FROM specification WHERE id = ?1",
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
                "SELECT id, id_speckle, version_number, id_source_range FROM specification WHERE id_speckle = ?1 ORDER BY version_number",
                [id_speckle],
            )
            .await?;
        let mut specifications = Vec::new();
        while let Some(row) = rows.next().await? {
            specifications.push(Specification::from_row(&row)?);
        }
        Ok(specifications)
    }

    pub async fn insert_implementation(
        &self,
        implementation: NewImplementation,
    ) -> Result<Implementation, DbError> {
        self.conn
            .execute(
                "INSERT INTO implementation (id_specification, id_source_range) VALUES (?1, ?2)",
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
                "SELECT id, id_specification, id_source_range FROM implementation WHERE id = ?1",
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
                "SELECT id, id_specification, id_source_range FROM implementation WHERE id_specification = ?1",
                [id_specification],
            )
            .await?;
        let mut implementations = Vec::new();
        while let Some(row) = rows.next().await? {
            implementations.push(Implementation::from_row(&row)?);
        }
        Ok(implementations)
    }
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
                version_number: 1,
                id_source_range: source_range.id,
            })
            .await?;
        assert_eq!(
            db.list_specifications_for_speckle(speckle.id).await?,
            vec![specification.clone()]
        );

        let implementation = db
            .insert_implementation(NewImplementation {
                id_specification: specification.id,
                id_source_range: source_range.id,
            })
            .await?;
        assert_eq!(
            db.list_implementations_for_specification(specification.id)
                .await?,
            vec![implementation]
        );

        Ok(())
    }
}
