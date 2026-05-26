use speckle_syntax::{Item, SourceRange};
use syn::{Attribute, Meta};

use super::speckle_visitor::SpeckleVisitor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifiedSpeckleItem {
    pub identifier: String,
    pub item_range: SourceRange,
    pub source_text: String,
}

pub fn find_identified_speckle_items(source: &str, file: &syn::File) -> Vec<IdentifiedSpeckleItem> {
    let mut visitor = SpeckleVisitor::new();
    visitor.visit_file(file);
    visitor
        .into_sites()
        .into_iter()
        .filter_map(|site| identified_item_from_site(source, &site.attrs, site.item_range))
        .collect()
}

fn identified_item_from_site(
    source: &str,
    attrs: &[Attribute],
    item_range: SourceRange,
) -> Option<IdentifiedSpeckleItem> {
    for attr in attrs {
        if let Some(identifier) = speckle_identifier(attr) {
            let item_source = &source[item_range.byte_start..item_range.byte_end];
            let item = syn::parse_str::<Item>(item_source)
                .expect("item source should parse after successful file parse");
            let source_text = item.to_string();
            return Some(IdentifiedSpeckleItem {
                identifier,
                item_range,
                source_text,
            });
        }
    }
    None
}

fn speckle_identifier(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("speckle") {
        return None;
    }

    let list = match &attr.meta {
        Meta::List(list) => list,
        _ => return None,
    };

    if let Ok(lit) = list.parse_args::<syn::LitStr>() {
        return Some(lit.value());
    }

    let mut identifier = None;
    let _ = list.parse_nested_meta(|meta| {
        if !meta.path.is_ident("identifier") {
            return Err(meta.error("expected `identifier`"));
        }
        let value = meta
            .value()?
            .parse::<syn::LitStr>()
            .map_err(|err| meta.error(err))?;
        identifier = Some(value.value());
        Ok(())
    });
    identifier
}
