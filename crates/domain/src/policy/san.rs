use std::net::IpAddr;

use ipnet::IpNet;

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
pub enum SanRule {
    DnsExact(String),

    /// Matches descendants, not the base domain itself.
    DnsSubdomainOf(String),

    IpRange(IpNet),

    Rfc822Exact(String),
    Rfc822Domain(String),

    UriExact(String),

    /// Intended for workload identities.
    Spiffe {
        trust_domain: String,
        path_prefix: String,
    },
}
