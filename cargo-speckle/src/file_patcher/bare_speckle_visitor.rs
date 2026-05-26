use proc_macro2::Span;

use super::speckle_visitor::SpeckleVisitor;

#[derive(Debug, Clone)]
pub struct BareSpeckleAttribute {
    pub range: Span,
}

pub fn find_bare_speckle_attributes(file: &syn::File) -> Vec<BareSpeckleAttribute> {
    let mut visitor = SpeckleVisitor::new();
    visitor.visit_file(file);
    visitor
        .into_sites()
        .into_iter()
        .filter(|site| site.attribute.kind.is_unidentified())
        .map(|site| BareSpeckleAttribute {
            range: site.attribute.span,
        })
        .collect()
}
