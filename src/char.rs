use unicode_width::UnicodeWidthChar;
use unicode_normalization::char::canonical_combining_class;

use scripts::Script;


pub trait CharExt {
    fn char_type(&self) -> CharType;
    fn script(&self) -> Option<Script>;
    fn is_multicolumn(&self) -> bool;
    fn is_combining(&self) -> bool;
}

impl CharExt for char {
    fn char_type(&self) -> CharType {
        if self.is_control() {
            CharType::Control
        }
        else if self.is_combining() {
            CharType::Combining
        }
        else {
            CharType::Normal
        }
    }

    fn is_combining(&self) -> bool {
        canonical_combining_class(*self) != 0
    }

    fn script(&self) -> Option<Script> {
        Script::lookup(*self)
    }

    fn is_multicolumn(&self) -> bool {
        UnicodeWidthChar::width(*self) == Some(2)
    }
}

#[derive(PartialEq, Debug)]
pub enum CharType {
    Normal,
    Combining,
    Control,
}
