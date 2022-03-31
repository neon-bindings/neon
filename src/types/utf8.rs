use std::borrow::Cow;

const SMALL_MAX: usize = std::i32::MAX as usize;

/// V8 APIs that take UTF-8 strings take their length in the form of 32-bit
/// signed integers. This type represents a UTF-8 string that contains no
/// more than `i32::MAX` bytes of code units.
pub struct SmallUtf8<'a> {
    contents: Cow<'a, str>,
}

impl<'a> SmallUtf8<'a> {
    pub fn lower(&self) -> (*const u8, i32) {
        (self.contents.as_ptr(), self.contents.len() as i32)
    }
}

/// A UTF-8 string that can be lowered to a representation usable for V8
/// APIs.
pub struct Utf8<'a> {
    contents: Cow<'a, str>,
}

impl<'a> From<&'a str> for Utf8<'a> {
    fn from(s: &'a str) -> Self {
        Utf8 {
            contents: Cow::from(s),
        }
    }
}

impl<'a> Utf8<'a> {
    pub fn size(&self) -> usize {
        self.contents.len()
    }

    pub fn into_small(self) -> Option<SmallUtf8<'a>> {
        if self.size() < SMALL_MAX {
            Some(SmallUtf8 {
                contents: self.contents,
            })
        } else {
            None
        }
    }

    pub fn into_small_unwrap(self) -> SmallUtf8<'a> {
        let size = self.size();
        self.into_small().unwrap_or_else(|| {
            panic!("{} >= i32::MAX", size);
        })
    }

    pub fn truncate(self) -> SmallUtf8<'a> {
        let size = self.size();
        let mut contents = self.contents;

        if size >= SMALL_MAX {
            let s: &mut String = contents.to_mut();
            s.truncate(SMALL_MAX - 3);
            s.push_str("...");
        }

        SmallUtf8 { contents }
    }
}
