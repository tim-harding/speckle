use quote::ToTokens;
use rkyv::rancor::Error as RkyvError;
use rkyv::{Archive, Deserialize, Serialize};
use rkyv::{access, deserialize, to_bytes};

use crate::item::Item;
use crate::speckle_attribute::SpeckleAttributeKind;

/// Supported Rust item kinds for a `#[speckle]`-annotated item.
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[rkyv(derive(Debug, PartialEq))]
pub enum ItemKind {
    Static,
    Const,
    Struct,
    Enum,
    Union,
    Fn,
    Trait,
    Impl,
    Macro,
    Mod,
}

/// A `#[speckle]` attribute argument in archived form.
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[rkyv(derive(Debug, PartialEq))]
pub enum SpeckleArgument {
    Identified(String),
    Unidentified,
}

/// Canonical zero-copy representation of a `#[speckle]`-annotated item.
///
/// Used for revision comparison and database storage instead of raw source text.
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[rkyv(derive(Debug, PartialEq))]
pub struct StoredItem {
    pub kind: ItemKind,
    pub speckle_arguments: Vec<SpeckleArgument>,
    /// Token string of the annotated item's span content (body, fields, etc.).
    pub content: String,
}

/// Token string substituted for the annotated item's span content.
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[rkyv(derive(Debug, PartialEq))]
pub struct StoredSpanContent {
    pub content: String,
}

#[derive(thiserror::Error, Debug)]
pub enum StoredArchiveError {
    #[error(transparent)]
    Rkyv(#[from] RkyvError),
}

impl StoredItem {
    pub fn from_item(item: &Item) -> Result<Self, crate::SyntaxError> {
        let attribute = item.speckle_attribute()?;
        Ok(Self {
            kind: ItemKind::from(item),
            speckle_arguments: todo!(),
            content: item_content(item),
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, StoredArchiveError> {
        Ok(to_bytes::<RkyvError>(self)?.into_vec())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StoredArchiveError> {
        let archived = access::<ArchivedStoredItem, RkyvError>(bytes)?;
        Ok(deserialize::<StoredItem, RkyvError>(archived)?)
    }
}

impl StoredSpanContent {
    pub fn to_bytes(&self) -> Result<Vec<u8>, StoredArchiveError> {
        Ok(to_bytes::<RkyvError>(self)?.into_vec())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StoredArchiveError> {
        let archived = access::<ArchivedStoredSpanContent, RkyvError>(bytes)?;
        Ok(deserialize::<StoredSpanContent, RkyvError>(archived)?)
    }
}

impl From<&Item> for ItemKind {
    fn from(item: &Item) -> Self {
        match item {
            Item::Static(_) => ItemKind::Static,
            Item::Const(_) => ItemKind::Const,
            Item::Struct(_) => ItemKind::Struct,
            Item::Enum(_) => ItemKind::Enum,
            Item::Union(_) => ItemKind::Union,
            Item::Fn(_) => ItemKind::Fn,
            Item::Trait(_) => ItemKind::Trait,
            Item::Impl(_) => ItemKind::Impl,
            Item::Macro(_) => ItemKind::Macro,
            Item::Mod(_) => ItemKind::Mod,
        }
    }
}

impl From<SpeckleAttributeKind> for SpeckleArgument {
    fn from(kind: SpeckleAttributeKind) -> Self {
        match kind {
            SpeckleAttributeKind::Identified(id) => SpeckleArgument::Identified(id),
            SpeckleAttributeKind::Unidentified => SpeckleArgument::Unidentified,
        }
    }
}
fn item_content(item: &Item) -> String {
    match item {
        Item::Static(item) => item.expr.to_token_stream().to_string(),
        Item::Const(item) => item.expr.to_token_stream().to_string(),
        Item::Struct(item) => item.fields.to_token_stream().to_string(),
        Item::Enum(item) => item.variants.to_token_stream().to_string(),
        Item::Union(item) => item.fields.to_token_stream().to_string(),
        Item::Fn(item) => item.block.to_token_stream().to_string(),
        Item::Trait(item) => tokens_to_string(&item.items),
        Item::Impl(item) => tokens_to_string(&item.items),
        Item::Macro(item) => item.mac.tokens.to_token_stream().to_string(),
        Item::Mod(item) => {
            let (_, items) = item
                .content
                .as_ref()
                .expect("file modules are rejected during parsing");
            tokens_to_string(items)
        }
    }
}

fn tokens_to_string<T: ToTokens>(items: &[T]) -> String {
    items
        .iter()
        .map(|item| item.to_token_stream().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use syn::parse_str;

    #[test]
    fn stored_item_round_trips_through_rkyv() {
        let item = parse_str::<Item>(indoc! {"
            #[speckle(\"deadbeef\")]
            fn foo() { 1 }
        "})
        .unwrap();
        let stored = StoredItem::from_item(&item).unwrap();
        let bytes = stored.to_bytes().unwrap();
        let decoded = StoredItem::from_bytes(&bytes).unwrap();
        assert_eq!(stored, decoded);
    }

    #[test]
    fn stored_item_captures_kind_and_content() {
        let item = parse_str::<Item>(indoc! {"
            #[speckle]
            fn foo() { 1 }
        "})
        .unwrap();
        let stored = StoredItem::from_item(&item).unwrap();
        assert_eq!(stored.kind, ItemKind::Fn);
        assert_eq!(stored.content, "{ 1 }");
    }
}
