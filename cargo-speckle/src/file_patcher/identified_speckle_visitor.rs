use proc_macro2::Span;
use speckle_syntax::Item;

use super::speckle_visitor::{SpeckleSite, SpeckleVisitor};

#[derive(Debug, Clone)]
pub struct IdentifiedSpeckleItem {
    pub identifier: String,
    pub item_range: Span,
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
    let item_bytes = site.item_range.byte_range();
    let item_source = &source[item_bytes.start..item_bytes.end];
    let item = syn::parse_str::<Item>(item_source)
        .expect("item source should parse after successful file parse");
    Some(IdentifiedSpeckleItem {
        identifier,
        item_range: site.item_range,
        source_text: item.to_string(),
    })
}
