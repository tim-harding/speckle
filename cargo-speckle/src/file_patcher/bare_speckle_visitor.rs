use speckle_syntax::SourceRange;

use super::speckle_visitor::SpeckleVisitor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BareSpeckleAttribute {
    pub range: SourceRange,
}

pub fn find_bare_speckle_attributes(file: &syn::File) -> Vec<BareSpeckleAttribute> {
    let mut visitor = SpeckleVisitor::new();
    visitor.visit_file(file);
    visitor
        .into_sites()
        .into_iter()
        .filter(|site| site.attribute.is_bare())
        .map(|site| BareSpeckleAttribute {
            range: SourceRange::from(site.attribute.span),
        })
        .collect()
}
