/// Utility methods for working with string cases.
pub trait StringCaseExt {
    /// Returns the titlecase equivalent of a string, as a new [`String`].
    fn to_titlecase(&self) -> String;
}

impl StringCaseExt for &str {
    fn to_titlecase(&self) -> String {
        let mut chars = self.chars();

        match chars.next() {
            None => String::new(),
            Some(char) => char.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

impl StringCaseExt for String {
    fn to_titlecase(&self) -> String {
        self.as_str().to_titlecase()
    }
}
