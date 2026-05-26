use speckle_syntax::SourceRange;
use syn::{Attribute, ImplItem, Meta, spanned::Spanned, visit::Visit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BareSpeckleAttribute {
    pub range: SourceRange,
}

pub fn find_bare_speckle_attributes(file: &syn::File) -> Vec<BareSpeckleAttribute> {
    let mut visitor = BareSpeckleVisitor { found: vec![] };
    for item in &file.items {
        visitor.visit_item(item);
    }
    visitor.found
}

fn is_bare_speckle(attr: &Attribute) -> bool {
    attr.path().is_ident("speckle") && matches!(attr.meta, Meta::Path(_))
}

struct BareSpeckleVisitor {
    found: Vec<BareSpeckleAttribute>,
}

impl BareSpeckleVisitor {
    fn check_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if is_bare_speckle(attr) {
                self.found.push(BareSpeckleAttribute {
                    range: SourceRange::from(attr.span()),
                });
            }
        }
    }
}

impl Visit<'_> for BareSpeckleVisitor {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        self.check_attrs(&node.attrs);
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                self.check_attrs(&method.attrs);
            }
        }
    }

    fn visit_item_mod(&mut self, node: &syn::ItemMod) {
        self.check_attrs(&node.attrs);
        if let Some((_, items)) = &node.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }

    fn visit_item_const(&mut self, node: &syn::ItemConst) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_static(&mut self, node: &syn::ItemStatic) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
        self.check_attrs(&node.attrs);
    }
}
