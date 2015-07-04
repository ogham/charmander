use unicode_width::UnicodeWidthChar;


pub trait CharExt {
    fn char_type(&self) -> CharType;
    fn is_multicolumn(&self) -> bool;
}

impl CharExt for char {
    fn char_type(&self) -> CharType {
        let num = *self as u32;

        if self.is_control() {
            CharType::Control
        }
        else if num >= 0x300 && num < 0x370 {
            CharType::Combining
        }
        else {
            CharType::Normal
        }
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
