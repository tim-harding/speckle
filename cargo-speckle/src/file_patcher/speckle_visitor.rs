use syn::{Attribute, visit::Visit};

fn is_speckle(attr: &Attribute) -> bool {
    attr.path().is_ident("speckle")
}

pub struct SpeckleVisitor {
    found: Vec<Attribute>,
}

impl SpeckleVisitor {
    fn push_speckle_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs.iter().filter(|attr| is_speckle(attr)) {
            self.found.push(attr.clone());
        }
    }
}

impl Visit<'_> for SpeckleVisitor {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        self.push_speckle_attrs(&node.attrs);
        for item in &node.items {
            self.visit_impl_item(item);
        }
    }

    fn visit_impl_item_fn(&mut self, node: &syn::ImplItemFn) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_mod(&mut self, node: &syn::ItemMod) {
        self.push_speckle_attrs(&node.attrs);
        if let Some((_, items)) = &node.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }

    fn visit_impl_item(&mut self, i: &'_ syn::ImplItem) {
        let attrs = match i {
            syn::ImplItem::Const(i) => Some(&i.attrs),
            syn::ImplItem::Fn(i) => Some(&i.attrs),
            syn::ImplItem::Type(i) => Some(&i.attrs),
            syn::ImplItem::Macro(i) => Some(&i.attrs),
            syn::ImplItem::Verbatim(_) => None,
            _ => None,
        };
        if let Some(attrs) = attrs {
            self.push_speckle_attrs(&attrs);
        }
    }

    fn visit_item_const(&mut self, node: &syn::ItemConst) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_static(&mut self, node: &syn::ItemStatic) {
        self.push_speckle_attrs(&node.attrs);
    }

    fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
        self.push_speckle_attrs(&node.attrs);
    }
}
