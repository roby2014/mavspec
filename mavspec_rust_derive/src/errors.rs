use crate::consts::MESSAGE_ID_MAX;

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    MessageId(#[from] MessageIdError),
    #[error(transparent)]
    CrcExtra(#[from] CrcExtraError),
    #[error(transparent)]
    Type(#[from] TypeError),
    #[error(transparent)]
    Field(#[from] FieldError),
    #[error(transparent)]
    Message(#[from] SpecError),
    #[error(transparent)]
    Enum(#[from] EnumError),
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum MessageIdError {
    #[error(
        "`Message` derive macro requires `message_id` attribute. For example: #[message_id(42)]"
    )]
    MissingAttribute,
    #[error("`message_id` should be a valid `u32` literal. For example: #[message_id(42)]")]
    NotALiteral,
    #[error("`message_id` should be an integer literal")]
    NotAnInteger,
    #[error("Error converting `message_id` into a valid `u32` literal: {0:?}")]
    InvalidU32Literal(#[from] syn::Error),
    #[error(
        "`message_id` should be a 24-bit unsigned integer value (0..{}), but {0} given",
        MESSAGE_ID_MAX
    )]
    OutOfBounds(u32),
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum CrcExtraError {
    #[error(
        "`Message` derive macro requires `crc_extra` attribute. For example: #[crc_extra(42)]"
    )]
    MissingAttribute,
    #[error("`crc_extra` should be a valid `u8` literal. For example: #[crc_extra(42)]")]
    NotALiteral,
    #[error("`crc_extra` should be an integer literal")]
    NotAnInteger,
    #[error("Error converting `crc_extra` into a valid `u8` literal: {0:?}")]
    InvalidU8Literal(#[from] syn::Error),
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum TypeError {
    #[error("invalid type: {0}")]
    Invalid(String),
    #[error("invalid array element type: {0}")]
    InvalidArrayElement(String),
    #[error("invalid scalar type: {0}")]
    InvalidScalar(String),
    #[error("type parsing error: {0}")]
    ParseError(syn::Error),
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum FieldError {
    #[error("can't parse `base_type` argument: {0}")]
    BaseTypeArgumentParseError(syn::Error),
    #[error("`base_type` argument should be a primitive numeric type like `u8` or `f32` but {0} was given")]
    InvalidBaseTypeArgument(String),
    #[error("enum representation should be an integer numeric type, field: `{0}`")]
    NonIntegerRepr(String),
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum SpecError {
    #[error("`Message` can be derived only from structs")]
    NotAStruct,
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum EnumError {
    #[error("`Enum` can be derived only from enums")]
    NotAnEnum,
    #[error("`Enum` can be applied to enums with numeric representation. Add `#[repr(<type>)]`.")]
    ReprIsMissing,
    #[error(
        "all variants should have explicit discriminants, but enum variant `{0}` is missing one"
    )]
    MissingDiscriminant(String),
}
