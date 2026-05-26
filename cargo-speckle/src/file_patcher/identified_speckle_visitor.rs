use speckle_syntax::{Item, SourceRange};
use syn::{Attribute, ImplItem, Meta, TraitItem, spanned::Spanned, visit::Visit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifiedSpeckleItem {
    pub identifier: String,
    pub item_range: SourceRange,
    pub source_text: String,
}

pub fn find_identified_speckle_items(source: &str, file: &syn::File) -> Vec<IdentifiedSpeckleItem> {
    let mut visitor = IdentifiedSpeckleVisitor {
        source,
        found: Vec::new(),
    };
    for item in &file.items {
        visitor.visit_item(item);
    }
    visitor.found
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

struct IdentifiedSpeckleVisitor<'source> {
    source: &'source str,
    found: Vec<IdentifiedSpeckleItem>,
}

impl IdentifiedSpeckleVisitor<'_> {
    fn check_item(&mut self, attrs: &[Attribute], item_span: impl Spanned) {
        for attr in attrs {
            if let Some(identifier) = speckle_identifier(attr) {
                let item_range = SourceRange::from(item_span.span());
                let item_source = &self.source[item_range.byte_start..item_range.byte_end];
                let item = syn::parse_str::<Item>(item_source)
                    .expect("item source should parse after successful file parse");
                let source_text = item.to_string();
                self.found.push(IdentifiedSpeckleItem {
                    identifier,
                    item_range,
                    source_text,
                });
                return;
            }
        }
    }
}

impl Visit<'_> for IdentifiedSpeckleVisitor<'_> {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        self.check_item(&node.attrs, node.span());
        for item in &node.items {
            if let TraitItem::Fn(method) = item {
                self.check_item(&method.attrs, method.span());
            }
        }
    }

    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        self.check_item(&node.attrs, node.span());
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                self.check_item(&method.attrs, method.span());
            }
        }
    }

    fn visit_item_mod(&mut self, node: &syn::ItemMod) {
        self.check_item(&node.attrs, node.span());
        if let Some((_, items)) = &node.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }

    fn visit_item_const(&mut self, node: &syn::ItemConst) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_static(&mut self, node: &syn::ItemStatic) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
        self.check_item(&node.attrs, node.span());
    }
}
