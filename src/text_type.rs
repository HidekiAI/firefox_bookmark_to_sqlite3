pub mod MangaModelTextType {
    use std::ops::Deref;

    // Define the custom trait
    pub trait MangaModelTextType {
        fn from_string(s: String) -> Self;
        fn from_str(s: &str) -> Self;
        fn to_string(&self) -> String;
        fn to_str(&self) -> &str;
    }

    // Implement the trait for String and &str
    impl MangaModelTextType for String {
        fn from_string(s: String) -> Self {
            s
        }

        fn from_str(s: &str) -> Self {
            s.to_string()
        }

        fn to_string(&self) -> String {
            self.clone()
        }

        fn to_str(&self) -> &str {
            self.as_str()
        }
    }

    impl MangaModelTextType for &str {
        fn from_string(s: String) -> Self {
            Self::from_str(&s)
        }

        fn from_str(s: &str) -> Self {
            // NOTE: Box::leak() converts a Box<T> into a &'static mut T with a static lifetime, leaking the box
            // this is higly undesirable, but for now, to shut up the compiler, we'll do it this way
            Box::leak(s.to_owned().into_boxed_str())
        }

        fn to_string(&self) -> String {
            MangaModelTextType::to_string(self).to_owned()
        }

        fn to_str(&self) -> &str {
            *self
        }
    }

    #[cfg(test)]
    mod tests {
        use super::MangaModelTextType;

        #[test]
        fn test_texttype_traits() {
            // Create instances using the trait methods
            let text1: String = MangaModelTextType::from_string("Hello".to_string());
            let text2: &str = MangaModelTextType::from_str("World");

            // Use the trait methods to obtain strings
            let s1: String = MangaModelTextType::to_string(&text1);
            let s2: &str = MangaModelTextType::to_str(&text2);

            // Verify the results
            assert_eq!(s1, "Hello");
            assert_eq!(s2, "World");
        }

        #[test]
        fn test_display() {
            let text1: String = MangaModelTextType::from_string("Hello".to_string());
            let text2: &str = MangaModelTextType::from_str("World");

            println!("Text 1: {}", text1);
            println!("Text 2: {}", text2);
        }
    }
}
