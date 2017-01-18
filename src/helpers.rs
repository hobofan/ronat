use syntax::ast;
use syntax::ast::{LitKind, MetaItemKind};

pub fn prepare_line(input: String) -> String {
    let mut result = input;

    // TODO: only replace if at line beginning
    result = result.replace("//! ", "");
    result = result.replace("//!", "");
    result = result.replace("/// ", "");
    result = result.replace("///", "");
    result = result.replace("// ", "");
    result = result.replace("//", "");

    result
}

pub fn to_text_block(attrs: &[ast::Attribute]) -> String {
    let maybe_lines: Vec<Option<String>> = attrs.iter().map(|attr|
        match attr.value.node {
            MetaItemKind::NameValue(ref spanned) => {
                match spanned.node {
                    LitKind::Str(text, _) => {
                        let mut test_text = text.as_str().to_string();
                        test_text = prepare_line(test_text);

                        return Some(test_text);
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    ).collect();

    let lines: Vec<_> = maybe_lines.into_iter().filter_map(|l| l).collect();
    let text_block = lines.join("\n");

    text_block
}
