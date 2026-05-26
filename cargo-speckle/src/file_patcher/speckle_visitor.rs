use proc_macro2::Span;
use speckle_syntax::SpeckleAttribute;
use syn::spanned::Spanned;
use syn::{Attribute, visit::Visit};

pub struct SpeckleSite {
    pub attribute: SpeckleAttribute,
    pub item_range: Span,
}

pub struct SpeckleVisitor {
    sites: Vec<SpeckleSite>,
}

impl SpeckleVisitor {
    pub fn new() -> Self {
        Self { sites: Vec::new() }
    }

    pub fn visit_file(&mut self, file: &syn::File) {
        for item in &file.items {
            self.visit_item(item);
        }
    }

    pub fn into_sites(self) -> Vec<SpeckleSite> {
        self.sites
    }

    fn push_speckle_attrs(&mut self, attrs: &[Attribute], span: impl Spanned) {
        let item_range = span.span();
        for attribute in attrs.iter().filter_map(|attr| SpeckleAttribute::parse(attr).ok()) {
            self.sites.push(SpeckleSite {
                attribute,
                item_range: item_range.clone(),
            });
        }
    }
}

impl Visit<'_> for SpeckleVisitor {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.push_speckle_attrs(&node.attrs, node);
    }

    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        self.push_speckle_attrs(&node.attrs, node);
    }

    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        self.push_speckle_attrs(&node.attrs, node);
    }

    fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        self.push_speckle_attrs(&node.attrs, node);
    }

    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        self.push_speckle_attrs(&node.attrs, node);
    }

    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        self.push_speckle_attrs(&node.attrs, node);
        for item in &node.items {
            self.visit_impl_item(item);
        }
    }

    fn visit_item_mod(&mut self, node: &syn::ItemMod) {
        self.push_speckle_attrs(&node.attrs, node);
        if let Some((_, items)) = &node.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }

    fn visit_impl_item(&mut self, i: &syn::ImplItem) {
        let attrs = match i {
            syn::ImplItem::Const(i) => Some(&i.attrs),
            syn::ImplItem::Fn(i) => Some(&i.attrs),
            syn::ImplItem::Type(i) => Some(&i.attrs),
            syn::ImplItem::Macro(i) => Some(&i.attrs),
            syn::ImplItem::Verbatim(_) => None,
            _ => None,
        };
        if let Some(attrs) = attrs {
            self.push_speckle_attrs(attrs, i);
        }
    }

    fn visit_item_const(&mut self, node: &syn::ItemConst) {
        self.push_speckle_attrs(&node.attrs, node);
    }

    fn visit_item_static(&mut self, node: &syn::ItemStatic) {
        self.push_speckle_attrs(&node.attrs, node);
    }

    fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
        self.push_speckle_attrs(&node.attrs, node);
    }
}
