//! Extension methods on `char` values.
//!
//! *Technically*, all of these methods don't need to be in a trait, and could
//! instead just be individual functions. They're only methods for aesthetics.

use unicode_normalization::char::canonical_combining_class;

use scripts::Script;


/// Extension methods on `char` values.
pub trait CharExt {

    /// How this character should be displayed.
    /// See the `DisplayType` enum for more details.
    fn char_type(&self) -> DisplayType;

    /// This character's script or writing system, if it has been associated
    /// with one.
    fn script(&self) -> Option<Script>;

    /// Whether this character is a Unicode combining character.
    fn is_combining(&self) -> bool;
}

impl CharExt for char {
    fn char_type(&self) -> DisplayType {
        if self.is_control() {
            DisplayType::Control
        }
        else if self.is_combining() {
            DisplayType::Combining
        }
        else {
            DisplayType::Normal
        }
    }

    fn is_combining(&self) -> bool {
        canonical_combining_class(*self) != 0
    }

    fn script(&self) -> Option<Script> {
        Script::lookup(*self)
    }
}


/// How to display a character.
#[derive(PartialEq, Debug)]
pub enum DisplayType {

    /// Nothing special about this character.
    Normal,

    /// This character is a combining character, and should be displayed with
    /// another symbol as its background.
    Combining,

    /// This character is a control character, and cannot be directly printed,
    /// so display its codepoint number instead.
    Control,
}
