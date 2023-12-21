use mavspec::protocol::MavType;

// See: https://doc.rust-lang.org/reference/keywords.html
const RUST_RESERVED_KEYWORDS: [&str; 50] = [
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "typeof",
    "virtual", "unsafe", "unsized", "use", "where", "while", "yield",
];

pub const MAX_COMMENT_LENGTH: usize = 80;

pub fn dialect_name(dialect_name: String) -> String {
    heck::AsSnakeCase(dialect_name).to_string()
}

pub fn enum_rust_name(enum_name: String) -> String {
    heck::AsUpperCamelCase(enum_name).to_string()
}

pub fn split_description(value: &str) -> Vec<String> {
    let mut result = "".to_string();
    let mut pos = 0;

    for (_, ch) in value.chars().enumerate() {
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

pub fn enum_mod_name(message_name: String) -> String {
    heck::AsSnakeCase(message_name).to_string()
}

pub fn enum_file_name(message_name: String) -> String {
    format!("{}.rs", enum_mod_name(message_name))
}

pub fn enum_entry_name(entry_name: String) -> String {
    heck::AsUpperCamelCase(entry_name).to_string()
}

pub fn message_mod_name(message_name: String) -> String {
    heck::AsSnakeCase(message_name).to_string()
}

pub fn message_file_name(message_name: String) -> String {
    format!("{}.rs", message_mod_name(message_name))
}

pub fn messages_enum_entry_name(message_name: String) -> String {
    message_struct_name(message_name)
}

pub fn message_struct_name(message_name: String) -> String {
    heck::AsUpperCamelCase(message_name).to_string()
}

pub fn message_raw_struct_name(message_name: String) -> String {
    format!("{}Raw", message_struct_name(message_name))
}

pub fn rust_var_name(var_name: String) -> String {
    let var_name = heck::AsSnakeCase(var_name).to_string();

    if RUST_RESERVED_KEYWORDS.contains(&var_name.as_str()) {
        return format!("r#{var_name}");
    }

    var_name
}

pub fn rust_default_value(mav_type: MavType) -> String {
    match mav_type {
        MavType::Array(inner, length) => {
            format!("[0{}; {}]", inner.rust_type(), length)
        }
        _ => format!("0{}", mav_type.rust_type()),
    }
}

pub fn t_bytes_read_fn(mav_type: MavType) -> String {
    match mav_type {
        MavType::Array(_, _) => "read_array".to_string(),
        _ => "read".to_string(),
    }
}

pub fn t_bytes_write_fn(mav_type: MavType) -> String {
    match mav_type {
        MavType::Array(_, _) => "write_array".to_string(),
        _ => "write".to_string(),
    }
}

pub fn module_path_to_crate_path(path: String) -> String {
    let mut parts = path.split("::").collect::<Vec<&str>>();
    parts[0] = "crate";
    parts.join("::")
}
