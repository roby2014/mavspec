use mavspec::protocol::MavType;

const MESSAGE_STRUCT_PREFIX: &str = "Msg";
const MESSAGE_STRUCT_SUFFIX: &str = "";
const MESSAGE_PROCESSOR_STRUCT_PREFIX: &str = "Msg";
const MESSAGE_PROCESSOR_STRUCT_SUFFIX: &str = "Processor";
// See: https://doc.rust-lang.org/reference/keywords.html
const RUST_RESERVED_KEYWORDS: [&str; 50] = [
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "typeof",
    "virtual", "unsafe", "unsized", "use", "where", "while", "yield",
];

pub fn dialect_name(dialect_name: String) -> String {
    heck::AsSnakeCase(dialect_name).to_string()
}

pub fn enum_rust_name(enum_name: String) -> String {
    heck::AsUpperCamelCase(enum_name).to_string()
}

pub fn message_mod_name(message_name: String) -> String {
    heck::AsSnakeCase(message_name).to_string()
}

pub fn message_file_name(message_name: String) -> String {
    format!("{}.rs", message_mod_name(message_name))
}

pub fn messages_enum_entry_name(message_name: String) -> String {
    heck::AsUpperCamelCase(message_name).to_string()
}

pub fn message_struct_name(message_name: String) -> String {
    format!(
        "{}{}{}",
        MESSAGE_STRUCT_PREFIX,
        heck::AsUpperCamelCase(message_name),
        MESSAGE_STRUCT_SUFFIX
    )
}

pub fn message_processor_struct_name(message_name: String) -> String {
    format!(
        "{}{}{}",
        MESSAGE_PROCESSOR_STRUCT_PREFIX,
        heck::AsUpperCamelCase(message_name),
        MESSAGE_PROCESSOR_STRUCT_SUFFIX
    )
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
