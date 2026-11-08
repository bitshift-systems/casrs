#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DistinguishedName {
    pub common_name: Option<String>,
    pub organization: Option<String>,
    pub organizational_unit: Option<String>,
    pub locality: Option<String>,
    pub state_or_province: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubjectConstraints {
    pub common_name: AttributeRule,
    pub organization: AttributeRule,
    pub organizational_unit: AttributeRule,
    pub locality: AttributeRule,
    pub state_or_province: AttributeRule,
    pub country: AttributeRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeRule {
    Forbidden,
    Optional(ValueRule),
    Required(ValueRule),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AttributeRuleError {
    #[error("Attribute is forbidden, but a value was provided.")]
    ForbiddenProvided,
    #[error("Attribute is required, but no value was provided.")]
    RequiredMissing,
    #[error(transparent)]
    ValueValidationFailed(#[from] ValueRuleError),
}

impl AttributeRule {
    pub fn validate(&self, value: Option<&str>) -> Result<(), AttributeRuleError> {
        match self {
            AttributeRule::Forbidden => {
                if value.is_some() {
                    return Err(AttributeRuleError::ForbiddenProvided);
                }
            }
            AttributeRule::Optional(rule) => {
                if let Some(v) = value {
                    rule.validate(v)?;
                }
            }
            AttributeRule::Required(rule) => {
                if let Some(v) = value {
                    rule.validate(v)?;
                } else {
                    return Err(AttributeRuleError::RequiredMissing);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueRule {
    AnyNonEmpty,
    Exact(String),
    OneOf(Vec<String>),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValueRuleError {
    #[error("Value must be non-empty, but an empty value was provided.")]
    NonEmptyRequired,

    #[error("Value must exactly match '{expected}', but found '{found}'.")]
    ExactMismatch { expected: String, found: String },

    #[error("Value must be one of the allowed set {allowed:?}, but found '{found}'.")]
    NotInAllowedSet { allowed: Vec<String>, found: String },
}

impl ValueRule {
    pub fn validate(&self, value: &str) -> Result<(), ValueRuleError> {
        match self {
            ValueRule::AnyNonEmpty => {
                if value.is_empty() {
                    return Err(ValueRuleError::NonEmptyRequired);
                }
            }
            ValueRule::Exact(expected) => {
                if value != expected {
                    return Err(ValueRuleError::ExactMismatch {
                        expected: expected.clone(),
                        found: value.to_string(),
                    });
                }
            }
            ValueRule::OneOf(allowed) => {
                if !allowed.contains(&value.to_string()) {
                    return Err(ValueRuleError::NotInAllowedSet {
                        allowed: allowed.clone(),
                        found: value.to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubjectPolicy {
    Empty,
    Fixed(DistinguishedName),
    Requested(SubjectConstraints),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SubjectPolicyError {
    #[error("Subject policy requires an empty subject, but a non-empty subject was provided.")]
    EmptySubjectProvided,

    #[error(
        "Subject policy requires a fixed subject, but the provided subject does not match. Expected: {expected:?}, Found: {found:?}"
    )]
    FixedSubjectMismatch {
        expected: DistinguishedName,
        found: DistinguishedName,
    },

    #[error("Subject policy constraint violation for attribute '{attribute}': {error:?}")]
    ConstraintViolation {
        attribute: String,
        error: AttributeRuleError,
    },
}

impl SubjectPolicy {
    pub fn validate(&self, subject: &DistinguishedName) -> Result<(), SubjectPolicyError> {
        match self {
            SubjectPolicy::Empty => {
                if subject.common_name.is_some()
                    || subject.organization.is_some()
                    || !subject.organizational_unit.is_some()
                    || subject.locality.is_some()
                    || subject.state_or_province.is_some()
                    || subject.country.is_some()
                {
                    return Err(SubjectPolicyError::EmptySubjectProvided);
                }
            }

            SubjectPolicy::Fixed(fixed) => {
                if subject != fixed {
                    return Err(SubjectPolicyError::FixedSubjectMismatch {
                        expected: fixed.clone(),
                        found: subject.clone(),
                    });
                }
            }

            SubjectPolicy::Requested(constraints) => {
                constraints
                    .common_name
                    .validate(subject.common_name.as_deref())
                    .map_err(|e| SubjectPolicyError::ConstraintViolation {
                        attribute: "common_name".to_string(),
                        error: e,
                    })?;

                constraints
                    .organization
                    .validate(subject.organization.as_deref())
                    .map_err(|e| SubjectPolicyError::ConstraintViolation {
                        attribute: "organization".to_string(),
                        error: e,
                    })?;

                constraints
                    .organizational_unit
                    .validate(subject.organizational_unit.as_deref())
                    .map_err(|e| SubjectPolicyError::ConstraintViolation {
                        attribute: "organizational_unit".to_string(),
                        error: e,
                    })?;

                constraints
                    .locality
                    .validate(subject.locality.as_deref())
                    .map_err(|e| SubjectPolicyError::ConstraintViolation {
                        attribute: "locality".to_string(),
                        error: e,
                    })?;

                constraints
                    .state_or_province
                    .validate(subject.state_or_province.as_deref())
                    .map_err(|e| SubjectPolicyError::ConstraintViolation {
                        attribute: "state_or_province".to_string(),
                        error: e,
                    })?;

                constraints
                    .country
                    .validate(subject.country.as_deref())
                    .map_err(|e| SubjectPolicyError::ConstraintViolation {
                        attribute: "country".to_string(),
                        error: e,
                    })?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case(ValueRule::AnyNonEmpty, "Common Name", None ; "when value takes any non-empty")]
    #[test_case(ValueRule::AnyNonEmpty, "", Some(ValueRuleError::NonEmptyRequired) ; "when value is empty")]
    #[test_case(ValueRule::Exact("ExactValue".to_string()), "ExactValue", None ; "when value matches exact")]
    #[test_case(ValueRule::Exact("ExactValue".to_string()), "WrongValue", Some(ValueRuleError::ExactMismatch { expected: "ExactValue".to_string(), found: "WrongValue".to_string() }) ; "when value does not match exact")]
    #[test_case(ValueRule::OneOf(vec!["Value1".to_string(), "Value2".to_string()]), "Value1", None ; "when value is in allowed set")]
    #[test_case(ValueRule::OneOf(vec!["Value1".to_string(), "Value2".to_string()]), "Value3", Some(ValueRuleError::NotInAllowedSet { allowed: vec!["Value1".to_string(), "Value2".to_string()], found: "Value3".to_string() }) ; "when value is not in allowed set")]
    fn test_value_rule_validation(rule: ValueRule, input: &str, expected: Option<ValueRuleError>) {
        let result = rule.validate(&input);
        if let Some(expected) = expected {
            assert!(result.is_err(), "expected error {:?}, but got Ok", expected);
            let err = result.unwrap_err();
            assert_eq!(
                err, expected,
                "expected error {:?}, got {:?}",
                expected, err
            );
        } else {
            assert!(result.is_ok(), "expected Ok, but got {:?}", result);
        }
    }

    #[test_case(AttributeRule::Forbidden, None, None ; "when forbidden attribute has value")]
    #[test_case(AttributeRule::Forbidden, Some("Value"), Some(AttributeRuleError::ForbiddenProvided) ; "when forbidden attribute has no value")]
    #[test_case(AttributeRule::Optional(ValueRule::AnyNonEmpty), None, None ; "when optional attribute is missing")]
    #[test_case(AttributeRule::Optional(ValueRule::AnyNonEmpty), Some("Value"), None ; "when optional attribute is present")]
    #[test_case(AttributeRule::Required(ValueRule::AnyNonEmpty), Some("Value"), None ; "when required attribute is present")]
    #[test_case(AttributeRule::Required(ValueRule::AnyNonEmpty), None, Some(AttributeRuleError::RequiredMissing) ; "when required attribute is missing")]
    #[test_case(AttributeRule::Required(ValueRule::Exact("Value".into())), Some("Not Value"), Some(AttributeRuleError::ValueValidationFailed(ValueRuleError::ExactMismatch { expected: "Value".into(), found: "Not Value".into() })) ; "value rule error surfaces correctly")]
    fn test_attribute_rule_validation(
        rule: AttributeRule,
        input: Option<&str>,
        expected: Option<AttributeRuleError>,
    ) {
        let result = rule.validate(input);
        if let Some(expected) = expected {
            assert!(result.is_err(), "expected error {:?}, but got Ok", expected);
            let err = result.unwrap_err();
            assert_eq!(
                err, expected,
                "expected error {:?}, got {:?}",
                expected, err
            );
        } else {
            assert!(result.is_ok(), "expected Ok, but got {:?}", result);
        }
    }

    #[test_case(SubjectPolicy::Empty, DistinguishedName { common_name: None, organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None }, None ; "when empty subject is valid")]
    #[test_case(SubjectPolicy::Empty, DistinguishedName { common_name: Some("CN".into()), organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None }, Some(SubjectPolicyError::EmptySubjectProvided) ; "when non-empty subject is invalid")]
    #[test_case(SubjectPolicy::Fixed(DistinguishedName { common_name: Some("CN".into()), organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None }), DistinguishedName { common_name: Some("CN".into()), organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None }, None ; "when fixed subject matches")]
    #[test_case(SubjectPolicy::Fixed(DistinguishedName { common_name: Some("CN".into()), organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None }), DistinguishedName { common_name: Some("WrongCN".into()), organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None }, Some(SubjectPolicyError::FixedSubjectMismatch { expected: DistinguishedName { common_name: Some("CN".into()), organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None }, found: DistinguishedName { common_name: Some("WrongCN".into()), organization: None, organizational_unit: None, locality: None, state_or_province: None, country: None } }) ; "when fixed subject does not match")]
    #[test_case(SubjectPolicy::Requested(SubjectConstraints { common_name: AttributeRule::Required(ValueRule::AnyNonEmpty), organization: AttributeRule::Optional(ValueRule::AnyNonEmpty), organizational_unit: AttributeRule::Forbidden, locality: AttributeRule::Optional(ValueRule::AnyNonEmpty), state_or_province: AttributeRule::Optional(ValueRule::AnyNonEmpty), country: AttributeRule::Optional(ValueRule::AnyNonEmpty) }), DistinguishedName { common_name: Some("CN".into()), organization: Some("Org".into()), organizational_unit: None, locality: None, state_or_province: None, country: None }, None ; "when requested subject is valid")]
    #[test_case(SubjectPolicy::Requested(SubjectConstraints { common_name: AttributeRule::Required(ValueRule::AnyNonEmpty), organization: AttributeRule::Optional(ValueRule::AnyNonEmpty), organizational_unit: AttributeRule::Forbidden, locality: AttributeRule::Optional(ValueRule::AnyNonEmpty), state_or_province: AttributeRule::Optional(ValueRule::AnyNonEmpty), country: AttributeRule::Optional(ValueRule::AnyNonEmpty) }), DistinguishedName { common_name: None, organization: Some("Org".into()), organizational_unit: None, locality: None, state_or_province: None, country: None }, Some(SubjectPolicyError::ConstraintViolation { attribute: "common_name".to_string(), error: AttributeRuleError::RequiredMissing }) ; "when requested subject is invalid")]
    fn test_subject_policy_validation(
        policy: SubjectPolicy,
        subject: DistinguishedName,
        expected: Option<SubjectPolicyError>,
    ) {
        let result = policy.validate(&subject);
        if let Some(expected) = expected {
            assert!(result.is_err(), "expected error {:?}, but got Ok", expected);
            let err = result.unwrap_err();
            assert_eq!(
                err, expected,
                "expected error {:?}, got {:?}",
                expected, err
            );
        } else {
            assert!(result.is_ok(), "expected Ok, but got {:?}", result);
        }
    }
}
