use rusqlite::{Result as SqliteResult, Row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speckle {
    pub id: i64,
    pub identifier: String,
}

impl Speckle {
    pub(crate) fn from_row(row: &Row<'_>) -> SqliteResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            identifier: row.get(1)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSpeckle {
    pub identifier: String,
}

impl NewSpeckle {
    pub(crate) fn into_params(self) -> [String; 1] {
        [self.identifier]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRange {
    pub id: i64,
    pub commit_hash: String,
    pub file_path: String,
    pub byte_start: i64,
    pub byte_end: i64,
}

impl SourceRange {
    pub(crate) fn from_row(row: &Row<'_>) -> SqliteResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            commit_hash: row.get(1)?,
            file_path: row.get(2)?,
            byte_start: row.get(3)?,
            byte_end: row.get(4)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSourceRange {
    pub commit_hash: String,
    pub file_path: String,
    pub byte_start: i64,
    pub byte_end: i64,
}

impl NewSourceRange {
    pub(crate) fn into_params(self) -> (String, String, i64, i64) {
        (
            self.commit_hash,
            self.file_path,
            self.byte_start,
            self.byte_end,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specification {
    pub id: i64,
    pub id_speckle: i64,
    pub id_source_range: i64,
    pub source_text: String,
}

impl Specification {
    pub(crate) fn from_row(row: &Row<'_>) -> SqliteResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            id_speckle: row.get(1)?,
            id_source_range: row.get(2)?,
            source_text: row.get(3)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSpecification {
    pub id_speckle: i64,
    pub id_source_range: i64,
    pub source_text: String,
}

impl NewSpecification {
    pub(crate) fn into_params(self) -> (i64, i64, String) {
        (self.id_speckle, self.id_source_range, self.source_text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationJob {
    pub id: i64,
    pub id_specification: i64,
    pub id_external: Option<String>,
}

impl ImplementationJob {
    pub(crate) fn from_row(row: &Row<'_>) -> SqliteResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            id_specification: row.get(1)?,
            id_external: row.get(2)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewImplementationJob {
    pub id_specification: i64,
    pub id_external: Option<String>,
}

impl NewImplementationJob {
    pub(crate) fn into_params(self) -> (i64, Option<String>) {
        (self.id_specification, self.id_external)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implementation {
    pub id: i64,
    pub id_specification: i64,
    pub id_implementation_job: Option<i64>,
    pub id_source_range: i64,
    pub source_tokens: Vec<u8>,
}

impl Implementation {
    pub(crate) fn from_row(row: &Row<'_>) -> SqliteResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            id_specification: row.get(1)?,
            id_implementation_job: row.get(2)?,
            id_source_range: row.get(3)?,
            source_tokens: row.get(4)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewImplementation {
    pub id_specification: i64,
    pub id_implementation_job: Option<i64>,
    pub id_source_range: i64,
    pub source_tokens: Vec<u8>,
}

impl NewImplementation {
    pub(crate) fn into_params(self) -> (i64, Option<i64>, i64, Vec<u8>) {
        (
            self.id_specification,
            self.id_implementation_job,
            self.id_source_range,
            self.source_tokens,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationAccepted {
    pub id_speckle: i64,
    pub id_implementation: i64,
}

impl ImplementationAccepted {
    pub(crate) fn from_row(row: &Row<'_>) -> SqliteResult<Self> {
        Ok(Self {
            id_speckle: row.get(0)?,
            id_implementation: row.get(1)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewImplementationAccepted {
    pub id_speckle: i64,
    pub id_implementation: i64,
}

impl NewImplementationAccepted {
    pub(crate) fn into_params(self) -> (i64, i64) {
        (self.id_speckle, self.id_implementation)
    }
}
