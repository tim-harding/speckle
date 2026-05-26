use speckle_syntax::SourceRange;
use syn::spanned::Spanned;
use syn::{Attribute, Meta};

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
        .flat_map(|site| site.attrs)
        .filter(|attr| is_bare_speckle(attr))
        .map(|attr| BareSpeckleAttribute {
            range: SourceRange::from(attr.span()),
        })
        .collect()
}

fn is_bare_speckle(attr: &Attribute) -> bool {
    attr.path().is_ident("speckle") && matches!(attr.meta, Meta::Path(_))
}
