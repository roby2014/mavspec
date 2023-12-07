use handlebars::{handlebars_helper, Handlebars};
use mavspec::protocol::MavType;

use super::conventions::{
    dialect_name, enum_rust_name, message_file_name, message_mod_name,
    message_processor_struct_name, message_struct_name, messages_enum_entry_name,
    module_path_to_crate_path, rust_default_value, rust_var_name, t_bytes_read_fn,
    t_bytes_write_fn,
};

handlebars_helper!(to_crate_path: | s: String | module_path_to_crate_path(s));
handlebars_helper!(to_dialect_name: | s: String | dialect_name(s));
handlebars_helper!(to_enum_rust_name: | s: String | enum_rust_name(s));
handlebars_helper!(to_message_mod_name: | s: String | message_mod_name(s));
handlebars_helper!(to_messages_enum_entry_name: | s: String | messages_enum_entry_name(s));
handlebars_helper!(to_message_file_name: | s: String | message_file_name(s));
handlebars_helper!(to_message_struct_name: | s: String | message_struct_name(s));
handlebars_helper!(to_message_processor_struct_name: | s: String | message_processor_struct_name(s));
handlebars_helper!(to_rust_var: | s: String | rust_var_name(s));
handlebars_helper!(to_rust_type: | mav_type: MavType | mav_type.rust_type().to_string());
handlebars_helper!(to_rust_default_value: | mav_type: MavType | rust_default_value(mav_type));
handlebars_helper!(to_reader_fn: | mav_type: MavType | t_bytes_read_fn(mav_type));
handlebars_helper!(to_writer_fn: | mav_type: MavType | t_bytes_write_fn(mav_type));

pub fn register_helpers(reg: &mut Handlebars) {
    reg.register_helper("to-crate-path", Box::new(to_crate_path));
    reg.register_helper("to-dialect-name", Box::new(to_dialect_name));
    reg.register_helper("to-enum-rust-name", Box::new(to_enum_rust_name));
    reg.register_helper("to-message-mod-name", Box::new(to_message_mod_name));
    reg.register_helper(
        "to-messages-enum-entry-name",
        Box::new(to_messages_enum_entry_name),
    );
    reg.register_helper("to-message-file-name", Box::new(to_message_file_name));
    reg.register_helper("to-message-struct-name", Box::new(to_message_struct_name));
    reg.register_helper(
        "to-message-processor-struct-name",
        Box::new(to_message_processor_struct_name),
    );
    reg.register_helper("to-rust-var", Box::new(to_rust_var));
    reg.register_helper("to-rust-type", Box::new(to_rust_type));
    reg.register_helper("to-rust-default-value", Box::new(to_rust_default_value));
    reg.register_helper("to-reader-fn", Box::new(to_reader_fn));
    reg.register_helper("to-writer-fn", Box::new(to_writer_fn));
}
