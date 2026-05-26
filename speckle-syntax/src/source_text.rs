use crate::{Item, SourceRange, SyntaxError};
use syn::parse_str;

pub fn minify_source_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn minified_content(source: &str) -> Result<String, SyntaxError> {
    let item = parse_str::<Item>(source).map_err(|err| SyntaxError::Parse(err.to_string()))?;
    let range = SourceRange::from(item.span_content());
    let content = &source[range.byte_start..range.byte_end];
    Ok(minify_source_text(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_source_text() {
        assert_eq!(minify_source_text("{ 1 }"), "{1}");
    }

    #[test]
    fn test_minified_content_fn() {
        assert_eq!(minified_content("fn foo() { 1 }").unwrap(), "{1}");
    }
}
