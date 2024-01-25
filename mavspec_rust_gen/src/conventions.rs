// See: https://doc.rust-lang.org/reference/keywords.html
const RUST_RESERVED_KEYWORDS: [&str; 50] = [
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "typeof",
    "virtual", "unsafe", "unsized", "use", "where", "while", "yield",
];
const RUST_RESERVED_IDENTIFIERS: [&str; 1] = ["TryFrom"];

pub const MAX_COMMENT_LENGTH: usize = 80;
pub const NUMERIC_IDENTIFIER_PREFIX: &str = "_";
pub const RUST_KEYWORD_POSTFIX: &str = "_";
pub const EMPTY_IDENT_REPLACEMENT: &str = "_";

pub fn dialect_mod_name(dialect_name: String) -> String {
    heck::AsSnakeCase(dialect_name).to_string()
}

pub fn valid_rust_name(name: String) -> String {
    if RUST_RESERVED_KEYWORDS.contains(&name.as_str())
        || RUST_RESERVED_IDENTIFIERS.contains(&name.as_str())
    {
        return format!("{name}{}", RUST_KEYWORD_POSTFIX);
    }

    let name = if name.is_empty() {
        EMPTY_IDENT_REPLACEMENT.to_string()
    } else {
        name
    };

    match name.chars().next() {
        Some(ch) if ch.is_numeric() => format!("{}{}", NUMERIC_IDENTIFIER_PREFIX, name),
        None | Some(_) => name,
    }
}

pub fn split_description(value: &str) -> Vec<String> {
    let mut result = "".to_string();
    let mut pos = 0;
    let value = value.replace('\t', " ");

    for ch in value.chars() {
        pos += 1;
        if pos >= MAX_COMMENT_LENGTH && ch == ' ' {
            pos = 0;
            result.push('\n');
        } else {
            result.push(ch);
        }
    }

    result.split('\n').map(|s| s.to_string()).collect()
}

pub fn enum_rust_name(enum_name: String) -> String {
    valid_rust_name(heck::AsUpperCamelCase(enum_name).to_string())
}

pub fn enum_mod_name(enum_name: String) -> String {
    valid_rust_name(heck::AsSnakeCase(enum_name).to_string())
}

pub fn enum_file_name(message_name: String) -> String {
    format!("{}.rs", enum_mod_name(message_name))
}

pub fn enum_entry_name(entry_name: String) -> String {
    valid_rust_name(heck::AsUpperCamelCase(entry_name).to_string())
}

pub fn enum_bitmask_entry_name(entry_name: String) -> String {
    valid_rust_name(entry_name)
}

pub fn message_mod_name(message_name: String) -> String {
    valid_rust_name(heck::AsSnakeCase(message_name).to_string())
}

pub fn message_file_name(message_name: String) -> String {
    format!("{}.rs", message_mod_name(message_name))
}

pub fn messages_enum_entry_name(message_name: &str) -> String {
    message_struct_name(message_name)
}

pub fn message_struct_name(message_name: &str) -> String {
    valid_rust_name(heck::AsUpperCamelCase(message_name).to_string())
}

pub fn rust_var_name(var_name: String) -> String {
    let var_name = heck::AsSnakeCase(var_name).to_string();
    valid_rust_name(var_name)
}
