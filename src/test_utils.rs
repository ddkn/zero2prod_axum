#[cfg(test)]
pub mod test_utils {
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    pub struct ValidEmailFixture(String);

    impl Arbitrary for ValidEmailFixture {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let username_strategy =
                "[a-zA-Z0-9._]{1,20}".prop_map(|s| s.to_string());
            let domain_strategy = "example";
            let tld_strategy = prop_oneof!["com", "net", "org"];

            (username_strategy, domain_strategy, tld_strategy)
                .prop_map(|(username, domain, tld)| {
                    let email = format!("{username}@{domain}.{tld}");
                    Self(email)
                })
                .boxed()
        }
    }

    impl AsRef<str> for ValidEmailFixture {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }
}
