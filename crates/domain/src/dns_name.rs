use std::{fmt, net::IpAddr, str::FromStr};

/// A validated DNS name that can be used in a certificate SAN.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DnsName(String);

/// Errors that can occur when validating a DNS name.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DnsNameError {
    #[error("DNS name cannot be empty")]
    Empty,

    #[error("invalid internationalised DNS name")]
    InvalidIdna,

    #[error("DNS name exceeds 253 characters")]
    TooLong,

    #[error("invalid DNS label: {0}")]
    InvalidLabel(String),

    #[error("IP addresses must use an IP SAN")]
    IpAddress,
}

impl DnsName {
    /// Creates a new validated DNS name from the given input string.
    pub fn new(input: &str) -> Result<Self, DnsNameError> {
        // Accept fully qualified names ending with a dot.
        let input = input.strip_suffix('.').unwrap_or(input);

        if input.is_empty() {
            return Err(DnsNameError::Empty);
        }

        // Convert Unicode names to ASCII A-labels.
        let ascii = idna::domain_to_ascii_strict(input)
            .map_err(|_| DnsNameError::InvalidIdna)?
            .to_ascii_lowercase();

        if ascii.len() > 253 {
            return Err(DnsNameError::TooLong);
        }

        // DNS SANs must not contain IP address literals.
        if ascii.parse::<IpAddr>().is_ok() {
            return Err(DnsNameError::IpAddress);
        }

        for label in ascii.split('.') {
            let valid = !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric();

            if !valid {
                return Err(DnsNameError::InvalidLabel(label.to_owned()));
            }
        }

        Ok(Self(ascii))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if this DNS name is a subdomain of the given parent DNS
    /// name.
    pub fn is_subdomain_of(&self, parent: &DnsName) -> bool {
        self.0
            .strip_suffix(parent.as_str())
            .is_some_and(|prefix| prefix.len() > 1 && prefix.ends_with('.'))
    }

    /// Returns true if this DNS name is equal to or a subdomain of the given
    /// parent DNS name.
    pub fn is_within(&self, parent: &DnsName) -> bool {
        self == parent || self.is_subdomain_of(parent)
    }
}

impl FromStr for DnsName {
    type Err = DnsNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for DnsName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("example.com", "example.com", false ; "no subdomain")]
    #[test_case("api.example.com", "example.com", true ; "first subdomain")]
    #[test_case("a.b.example.com", "example.com", true ; "deep subdomain")]
    #[test_case("notexample.com", "example.com", false ; "first subdomain for the wrong domain")]
    #[test_case("api.notexample.com", "example.com", false ; "deep subdomain for the wrong domain")]
    fn test_is_subdomain_of(input: &str, domain: &str, should_succeed: bool) {
        let input = DnsName::new(input).expect("Could not parse input DNS name");
        let domain = DnsName::new(domain).expect("Could not parse domain DNS name");
        assert!(input.is_subdomain_of(&domain) == should_succeed);
    }

    #[test_case("example.com", "example.com", true ; "top-level domain")]
    #[test_case("api.example.com", "example.com", true ; "subdomain")]
    #[test_case("notexample.com", "example.com", false ; "wrong domain")]
    fn test_is_within(input: &str, domain: &str, should_succeed: bool) {
        let input = DnsName::new(input).expect("Could not parse input DNS name");
        let domain = DnsName::new(domain).expect("Could not parse domain DNS name");
        assert!(input.is_within(&domain) == should_succeed);
    }
}
