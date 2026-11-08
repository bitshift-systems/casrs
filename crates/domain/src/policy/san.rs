use std::net::IpAddr;

use ipnet::IpNet;

use crate::dns_name::{DnsName, DnsNameError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectAltName {
    DnsName(String),
    IpAddress(IpAddr),
    Rfc822Name(String),
    Uri(String),
}

#[derive(Debug, Clone)]
pub enum SanPolicy {
    None,
    Fixed(Vec<SubjectAltName>),
    Requested(SanConstraints),
}

#[derive(Debug, Clone)]
pub struct SanConstraints {
    pub min_names: usize,
    pub max_names: usize,

    pub allow_wildcard_dns: bool,

    /// Every requested SAN must match at least one rule.
    pub allowed: Vec<SanRule>,
}

#[derive(Debug, Clone)]
pub enum DomainScope {
    Exact,
    SubdomainsOnly,
    ExactAndSubdomains,
}

impl DomainScope {
    pub fn validate(&self, input: &DnsName, expected: &DnsName) -> bool {
        match self {
            DomainScope::Exact => input == expected,
            DomainScope::SubdomainsOnly => input.is_subdomain_of(expected),
            DomainScope::ExactAndSubdomains => input.is_within(expected),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SanRule {
    DnsExact(DnsName),
    DnsSubdomainOf(DnsName),

    IpRange(IpNet),

    EmailExact(String),
    EmailDomain {
        domain: DnsName,
        scope: DomainScope,
    },

    UriExact(String),

    /// Intended for workload identities.
    Spiffe {
        trust_domain: String,
        path_prefix: String,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SanRuleError {
    #[error("DNS name does not match the expected value. Expected: {expected}, found: {found}")]
    DnsExactMismatch { expected: String, found: String },

    #[error(
        "DNS name does not match the expected subdomain. Expected subdomain of: {expected}, found: {found}"
    )]
    DnsSubdomainMismatch { expected: String, found: String },

    #[error(transparent)]
    InvalidDnsName(#[from] DnsNameError),

    #[error("IP address {0} is malformed or not in the expected range.")]
    IpMalformed(String),

    #[error("IP address {found} is not in the expected range: {expected}")]
    IpNotInRange { expected: String, found: String },

    #[error("Email does not match the expected value. Expected: {expected}, found: {found}")]
    EmailExactMismatch { expected: String, found: String },

    #[error(
        "Email domain name does not match the expected domain. Expected: {expected}, found: {found}"
    )]
    EmailDomainMismatch { expected: String, found: String },
}

impl SanRule {
    pub fn validate(&self, input: &str) -> Result<(), SanRuleError> {
        match self {
            Self::DnsExact(expected) => {
                let input = DnsName::new(input)?;
                if &input != expected {
                    return Err(SanRuleError::DnsExactMismatch {
                        expected: expected.to_string(),
                        found: input.to_string(),
                    });
                }
            }

            Self::DnsSubdomainOf(expected) => {
                let input = DnsName::new(input)?;
                if !input.is_subdomain_of(expected) {
                    return Err(SanRuleError::DnsExactMismatch {
                        expected: expected.to_string(),
                        found: input.to_string(),
                    });
                }
            }

            Self::IpRange(range) => {
                if let Ok(ip) = input.parse::<IpAddr>() {
                    if !range.contains(&ip) {
                        return Err(SanRuleError::IpNotInRange {
                            expected: range.to_string(),
                            found: input.to_string(),
                        });
                    }
                } else {
                    return Err(SanRuleError::IpMalformed(input.to_string()));
                }
            }

            Self::EmailExact(expected) => {
                if input != expected {
                    return Err(SanRuleError::EmailExactMismatch {
                        expected: expected.to_string(),
                        found: input.to_string(),
                    });
                }
            }

            Self::EmailDomain { domain, scope } => {
                let parts: Vec<&str> = input.split('@').collect();
                if parts.len() != 2 {
                    panic!("FIXME!!! Invalid email address: {input:?}");
                }
                let input = DnsName::new(parts[1])?;

                if !scope.validate(&input, domain) {
                    return Err(SanRuleError::EmailDomainMismatch {
                        expected: domain.to_string(),
                        found: input.to_string(),
                    });
                }
            }

            _ => unimplemented!(),
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use super::*;
    use test_case::test_case;

    #[test_case(SanRule::DnsExact(DnsName::new("example.com").unwrap()), "example.com", None ; "exact match")]
    #[test_case(SanRule::DnsExact(DnsName::new("example.com").unwrap()), "api.example.com", Some(SanRuleError::DnsExactMismatch { expected: "example.com".to_string(), found: "api.example.com".to_string() }) ; "exact mismatch")]
    #[test_case(SanRule::DnsSubdomainOf(DnsName::new("example.com").unwrap()), "api.example.com", None ; "subdomain match")]
    #[test_case(SanRule::DnsSubdomainOf(DnsName::new("example.com").unwrap()), "example.com", Some(SanRuleError::DnsExactMismatch { expected: "example.com".to_string(), found: "example.com".to_string() }) ; "subdomain mismatch")]
    #[test_case(SanRule::IpRange(IpNet::new(Ipv4Addr::new(10,0,0,0).into(), 24).unwrap()), "10.0.0.15", None ; "ip range match")]
    #[test_case(SanRule::IpRange(IpNet::new(Ipv4Addr::new(10,0,0,0).into(), 24).unwrap()), "10.1.0.15", Some(SanRuleError::IpNotInRange { expected: "10.0.0.0/24".to_string(), found: "10.1.0.15".to_string() }) ; "ip range mismatch")]
    #[test_case(SanRule::IpRange(IpNet::new(Ipv4Addr::new(10,0,0,0).into(), 24).unwrap()), "not.an.ip", Some(SanRuleError::IpMalformed("not.an.ip".to_string())) ; "ip malformed")]
    #[test_case(SanRule::EmailExact("jdoe@example.com".to_string()), "jdoe@example.com", None ; "email exact match")]
    #[test_case(SanRule::EmailExact("jdoe@example.com".to_string()), "jappleseed@example.com", Some(SanRuleError::EmailExactMismatch { expected: "jdoe@example.com".to_string(), found: "jappleseed@example.com".to_string() }) ; "email exact mismatch")]
    #[test_case(SanRule::EmailDomain { domain: DnsName::new("example.com").unwrap(), scope: DomainScope::Exact }, "jdoe@example.com", None ; "email domain exact match")]
    #[test_case(SanRule::EmailDomain { domain: DnsName::new("example.com").unwrap(), scope: DomainScope::Exact }, "jdoe@hello.example.com", Some(SanRuleError::EmailDomainMismatch { expected: "example.com".to_string(), found: "hello.example.com".to_string() }) ; "email domain exact mismatch")]
    #[test_case(SanRule::EmailDomain { domain: DnsName::new("example.com").unwrap(), scope: DomainScope::ExactAndSubdomains }, "jdoe@example.com", None ; "email domain exact and subdomains match")]
    #[test_case(SanRule::EmailDomain { domain: DnsName::new("example.com").unwrap(), scope: DomainScope::ExactAndSubdomains }, "jdoe@hello.example.com", None ; "email domain exact and subdomains match with subdomain")]
    #[test_case(SanRule::EmailDomain { domain: DnsName::new("example.com").unwrap(), scope: DomainScope::ExactAndSubdomains }, "jdoe@a.hello.example.com", None ; "email domain exact and subdomains match with deep subdomain")]
    #[test_case(SanRule::EmailDomain { domain: DnsName::new("example.com").unwrap(), scope: DomainScope::ExactAndSubdomains }, "jdoe@notexample.com", Some(SanRuleError::EmailDomainMismatch { expected: "example.com".to_string(), found: "notexample.com".to_string() }) ; "email domain exact and subdomains mismatch")]
    #[test_case(SanRule::EmailDomain { domain: DnsName::new("example.com").unwrap(), scope: DomainScope::ExactAndSubdomains }, "jdoe@hello.notexample.com", Some(SanRuleError::EmailDomainMismatch { expected: "example.com".to_string(), found: "hello.notexample.com".to_string() }) ; "email domain exact and subdomains mismatch with subdomain")]
    fn test_san_rule_validation(rule: SanRule, input: &str, expected: Option<SanRuleError>) {
        let result = rule.validate(input);
        if let Some(expected_error) = expected {
            let err = result.unwrap_err();
            assert_eq!(
                err, expected_error,
                "Expected {expected_error:?} but got {err:?}"
            );
        } else {
            assert!(
                result.is_ok(),
                "Got unexpected error: {:?}",
                result.unwrap_err()
            );
        }
    }
}
