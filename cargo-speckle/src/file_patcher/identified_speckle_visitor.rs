use speckle_syntax::{Item, SourceRange};

use super::speckle_visitor::{SpeckleSite, SpeckleVisitor};

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
        .filter_map(|site| identified_item_from_site(source, site))
        .collect()
}

fn identified_item_from_site(source: &str, site: SpeckleSite) -> Option<IdentifiedSpeckleItem> {
    let identifier = site.attribute.identifier()?.to_string();
    let item_source = &source[site.item_range.byte_start..site.item_range.byte_end];
    let item = syn::parse_str::<Item>(item_source)
        .expect("item source should parse after successful file parse");
    Some(IdentifiedSpeckleItem {
        identifier,
        item_range: site.item_range,
        source_text: item.to_string(),
    })
}
