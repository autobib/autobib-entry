use serde::de::{self, Deserializer, Error, Unexpected};

impl<'de> de::Deserialize<'de> for super::EntryType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut inner = String::deserialize(deserializer)?;

        if !inner.is_ascii() {
            return Err(D::Error::invalid_value(
                Unexpected::Str(&inner),
                &"an entry type composed of ASCII characters",
            ));
        }

        inner.make_ascii_lowercase();

        // SAFETY: `inner` is only accepted by the serde_bibtex deserialize impl if either it is
        // composed of non-ASCII characters, or ASCII characters which satisfy the field key rules
        // or also possibly capitals `A..=Z`. Therefore we only need to check that it is ASCII, and
        // convert any possible capitals to ASCII lowercase.
        Ok(Self(inner))
    }
}

impl<'de> de::Deserialize<'de> for super::FieldKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut inner = String::deserialize(deserializer)?;

        if !inner.is_ascii() {
            return Err(D::Error::invalid_value(
                Unexpected::Str(&inner),
                &"a field key composed of ASCII characters",
            ));
        }

        inner.make_ascii_lowercase();

        // SAFETY: `inner` is only accepted by the serde_bibtex deserialize impl if either it is
        // composed of non-ASCII characters, or ASCII characters which satisfy the field key rules
        // or also possibly capitals `A..=Z`. Therefore we only need to check that it is ASCII, and
        // convert any possible capitals to ASCII lowercase.
        Ok(Self(inner))
    }
}

impl<'de> de::Deserialize<'de> for super::FieldValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = String::deserialize(deserializer)?;

        // SAFETY: we do not check for the 'balanced `{}`' rule here because this rule is
        // automatically checked when parsing bibtex
        Ok(Self(inner))
    }
}
