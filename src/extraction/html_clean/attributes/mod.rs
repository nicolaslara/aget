mod comments;
mod empty;
mod images;
mod pruning;
mod text;

pub(in crate::extraction) use self::comments::remove_owned_comments;
pub(in crate::extraction) use self::empty::remove_owned_empty_elements;
pub(in crate::extraction) use self::images::clean_owned_base64_image_sources;
pub(in crate::extraction) use self::pruning::prune_owned_unwanted_attributes;
pub(in crate::extraction) use self::text::replace_owned_only_text_elements;
