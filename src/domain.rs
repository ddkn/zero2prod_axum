use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        // Grapheme is defined by the unicode standard as a "user-percieved"
        // character: `ö` is a single grapheme, but composed of two
        // characeters o(`o` and `¨`)
        //
        // `.grapheme(true)` returns an iterator that uses the extended
        // grapheme definition set, the recommended one
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters =
            ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace
            || is_too_long
            || contains_forbidden_characters
        {
            panic!("{} is not a valid subscriber name", s);
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
