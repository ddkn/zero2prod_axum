use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> SubscriberName {
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

        Self(s)
    }
    pub fn inner_ref(&self) -> &str {
        // The caller gets a shared reference to the inner string.
        // This gives the caller **read-only** access,
        // they have no way to compromise our invariants!
        &self.0
    }
}
