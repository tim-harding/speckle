use limbo::{Row, Value};

use crate::DbError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speckle {
    pub id: i64,
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSpeckle {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRange {
    pub id: i64,
    pub commit_hash: String,
    pub file_path: String,
    pub byte_start: i64,
    pub byte_end: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSourceRange {
    pub commit_hash: String,
    pub file_path: String,
    pub byte_start: i64,
    pub byte_end: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specification {
    pub id: i64,
    pub id_speckle: i64,
    pub version_number: i64,
    pub id_source_range: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSpecification {
    pub id_speckle: i64,
    pub version_number: i64,
    pub id_source_range: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implementation {
    pub id: i64,
    pub id_specification: i64,
    pub id_source_range: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewImplementation {
    pub id_specification: i64,
    pub id_source_range: i64,
}

impl Speckle {
    pub(crate) fn from_row(row: &Row) -> Result<Self, DbError> {
        Ok(Self {
            id: column_integer(row, 0, "speckle.id")?,
            identifier: column_text(row, 1, "speckle.identifier")?,
        })
    }
}

impl SourceRange {
    pub(crate) fn from_row(row: &Row) -> Result<Self, DbError> {
        Ok(Self {
            id: column_integer(row, 0, "source_range.id")?,
            commit_hash: column_text(row, 1, "source_range.commit_hash")?,
            file_path: column_text(row, 2, "source_range.file_path")?,
            byte_start: column_integer(row, 3, "source_range.byte_start")?,
            byte_end: column_integer(row, 4, "source_range.byte_end")?,
        })
    }
}

impl Specification {
    pub(crate) fn from_row(row: &Row) -> Result<Self, DbError> {
        Ok(Self {
            id: column_integer(row, 0, "specification.id")?,
            id_speckle: column_integer(row, 1, "specification.id_speckle")?,
            version_number: column_integer(row, 2, "specification.version_number")?,
            id_source_range: column_integer(row, 3, "specification.id_source_range")?,
        })
    }
}

impl Implementation {
    pub(crate) fn from_row(row: &Row) -> Result<Self, DbError> {
        Ok(Self {
            id: column_integer(row, 0, "implementation.id")?,
            id_specification: column_integer(row, 1, "implementation.id_specification")?,
            id_source_range: column_integer(row, 2, "implementation.id_source_range")?,
        })
    }
}

pub(crate) fn column_integer(row: &Row, index: usize, name: &str) -> Result<i64, DbError> {
    match row.get_value(index)? {
        Value::Integer(value) => Ok(value),
        other => Err(DbError::UnexpectedValue(format!(
            "expected integer for {name}, got {other:?}"
        ))),
    }
}

pub(crate) fn column_text(row: &Row, index: usize, name: &str) -> Result<String, DbError> {
    match row.get_value(index)? {
        Value::Text(value) => Ok(value),
        other => Err(DbError::UnexpectedValue(format!(
            "expected text for {name}, got {other:?}"
        ))),
    }
}
