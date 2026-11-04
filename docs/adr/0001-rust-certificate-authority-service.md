---
title: "ADR-0001: A Rust-based Certificate Authority service"
sidebar:
  label: ADR-0001 - Rust CAS
---

- **Status:** Proposed
- **Date:** 2026-11-04
- **Inspired by:** [AWS Private CA](https://aws.amazon.com/private-ca/), [Google CAS](https://cloud.google.com/security/products/certificate-authority-service) and [OpenBao/Vault](https://openbao.org/docs/secrets/pki/)

> Scope: this ADR covers **CA provisioning, CA families, hierarchies, user
> experience, and high-level design choices**. It deliberately does not specify
> Rust module layout or other implementation mechanics.

---

## 1. Context

### 1.1 Motivation and scope

We want a reusable, open-source service for operating private PKI in Rust.
The motivation is to make multi-tier CA hierarchies available without paid
feature gates or the operational complexity of a broader secrets platform.
This service is not intended to replace publicly trusted CA software.

### 1.2 Initial workflow

The first useful workflow is to declare a CA family, generate a root, generate
three intermediates, and issue a certificate from one intermediate. Operators
also need to bring existing CAs into the service and retrieve issued certificates
and their chains programmatically.

---

## 2. Decision

### 2.1 Service interface

Build a Rust service with an HTTP API and a CLI backed by that API. The initial
scope is one administrative domain; tenant isolation is outside this design.
A web UI is planned for later.

### 2.2 CA families and naming

Organise CAs into named **families** that persist across root rotation. Each
family has independent root and intermediate naming series. For example,
`Bitshift Certificate Authority` suggests `Bitshift Certificate Authority R1`
and `Bitshift Certificate Authority I1`. Operators may supply custom names.
Names are not CA identities, and intermediate numbering spans the family
rather than restarting for each root.

### 2.3 Hierarchy support

Support nested intermediate CAs, with a configurable maximum path length on
both roots and intermediates. An intermediate with a limit of zero cannot
issue child intermediates. A hierarchy can therefore include a root, a general
intermediate, and a PostgreSQL mTLS intermediate beneath it.

### 2.4 CA rotation and lifecycle

Old roots and intermediates remain available when a new root is introduced.
Rotation permits coexistence rather than automatically replacing the existing
hierarchy.
Operators can disable and re-enable a CA for new issuance. Disabling issuance
does not revoke existing certificates or remove the CA and its chain from retrieval.

### 2.5 Security and key management

Use token-based authentication. All authenticated callers are administrators
in v1; finer-grained authorisation is deferred, with room to layer it on later.

Manage generated and imported CA keys through a replaceable **key provider**
that supports signing. The initial provider stores encrypted software keys
and uses an operator-supplied unlocking secret. Private key material must not
be persisted in plaintext. Provider selection is configuration; Rust module
layout and the exact cryptographic construction are outside this ADR.

The service starts locked after every restart. An operator unlocks it through
the CLI. Authenticated certificate retrieval remains available while locked,
but signing, key import, and key generation are unavailable. Encrypted backups
require separately retained recovery material; losing all means of unlocking
the keys makes them unrecoverable.

### 2.6 Import and export

Support private-key import through PEM bundles, but do not provide private-key
export in v1. Reconstruct imported chains, retaining ancestors without supplied
keys as certificate-only CAs. Their signing capability can be enabled by
subsequently importing the matching key.

### 2.7 Certificate issuance and retrieval

Accept caller-supplied CSRs for leaf issuance. Track issued certificates and
provide certificate-chain retrieval through both the API and CLI.
Named issuance profiles constrain permitted identities, certificate usages,
and maximum lifetimes; a CSR requests values rather than granting permission
to issue them.
Profiles are reusable across CAs. Operators explicitly assign the profiles a
CA permits, and each issuance request explicitly selects one. Profile constraints
apply to administrator requests too, although administrators can edit profiles.
Reject explicitly requested lifetimes that exceed the profile's maximum or
the issuing CA's remaining validity, explaining the permitted limit rather
than silently shortening the certificate's lifetime.

### 2.8 Revocation and future protocols

Record certificate revocations in v1, including their reason and time, and
retain issued-certificate records and issuer relationships for future publishers.
CRLs, OCSP, and ACME are deferred. Recording a revocation does not notify relying
parties; the API and CLI must make this limitation explicit until publication
support exists.

---

## 3. Data/API design

### 3.1 Domain concepts

The agreed domain concepts are:

| Concept            | Meaning                                                                                                          |
| ------------------ | ---------------------------------------------------------------------------------------------------------------- |
| CA family          | Persistent grouping with independent root and intermediate naming series.                                        |
| CA                 | A root or intermediate, with its own identity, name, certificate, hierarchy relationship, and path-length limit. |
| Key provider       | Configured component responsible for CA key custody and signing.                                                 |
| Issuance profile   | Named policy governing permitted identities, usages, and certificate lifetime.                                   |
| Issued certificate | A tracked certificate issued from a caller-supplied CSR, associated with its issuing CA.                         |
| Certificate chain  | Certificates establishing the issuer path, retrievable through the API and CLI.                                  |
| Revocation record  | A recorded certificate revocation with its reason and time, distinct from publication to relying parties.        |

### 3.2 API capabilities

The API must support family creation, root and intermediate creation, PEM-bundle
import, issuance-profile management, CSR-based issuance, reversible issuance
disabling, revocation recording, and retrieval of certificates and their chains.
Certificate-only CAs must be distinguishable
from CAs with signing keys. Private-key export is not an API capability in v1.
Exact routes, request schemas, identifiers, and chain response formats remain
to be decided.

---

## 4. Key behaviours

### 4.1 Naming and hierarchy constraints

- Suggest sequential CA names while accepting operator-supplied names.
- Allow intermediate CAs to issue further intermediates within the permitted
  hierarchy depth; a configured limit of zero prevents child-CA issuance.

X.509 path-length constraints count non-self-issued intermediate certificates;
leaf certificates do not consume this allowance. The service's issuance policy
must enforce the configured hierarchy limit as well as produce appropriate
certificate constraints. See RFC 5280, section 4.2.1.9.

### 4.2 Key generation and import

- Generate CA keys within the service's key-provider boundary, or accept
  existing private key material through PEM import.
- Accept a PEM bundle containing a private key and certificate chain,
  validate that the key matches its certificate, and reconstruct the corresponding
  CA and chain data in the service. Ancestors without keys are certificate-only
  CAs and cannot sign. A later matching-key import enables signing on the
  existing CA record.

### 4.3 Security and locked operation

- Authenticate API callers using tokens, with administrator access for all
  authenticated callers in v1.
- Protect imported keys under the configured provider before persistence.
  Application storage, temporary files, and logs must not persist plaintext
  private key material.
- Require an unlocking secret for the initial software-key provider. Secret
  delivery and the detailed recovery procedure remain to be specified. Start
  locked after every restart; allow certificate retrieval while locked, but
  reject signing, key import, and key generation until an operator unlocks it.

### 4.4 Issuance policy and retrieval

- Issue leaf certificates from caller-supplied CSRs and retain records that
  support subsequent certificate and chain retrieval.
- Enforce the selected issuance profile when evaluating certificate requests.
  Require an explicit profile selection and verify that the issuing CA permits
  that profile. Administrator status does not bypass profile constraints.
- Reject explicit lifetime requests exceeding the profile maximum or issuer's
  remaining validity, with an error explaining the permitted maximum.

### 4.5 Rotation and issuance control

- Retain old roots and intermediates alongside new roots during rotation.
- Allow operators to disable and re-enable new issuance on a CA while retaining
  existing certificates and chains. Re-enabling issuance does not bypass other
  issuance constraints, supply a missing key, or unlock the key provider.

### 4.6 Revocation recording

- Record revocation reason and time while retaining the certificate and its
  issuer relationship. Clearly distinguish recording from CRL/OCSP publication.

---

## 5. Usage walkthroughs

### 5.1 Create a family and issue a certificate

1. Unlock the service's software-key provider.
2. Create the `Bitshift Certificate Authority` family.
3. Create its root, accepting the suggested name
   `Bitshift Certificate Authority R1` and choosing its path-length limit.
4. Create three intermediates under the root, accepting the suggested names
   `Bitshift Certificate Authority I1`, `I2`, and `I3` with the same family prefix.
5. Define an issuance profile, assign it to one intermediate, and submit a
   caller-generated CSR to that intermediate with the profile explicitly selected.
6. Retrieve the issued certificate and its chain through the CLI or HTTP API.

### 5.2 Import an existing CA

1. Supply a PEM bundle containing the CA private key and certificate chain.
2. Validate that the private key matches the CA certificate, reconstruct the
   CA's relationship to its chain, and associate it with a family. Represent
   ancestors without keys as certificate-only CAs.
3. Protect the private key with the configured provider before storing it.
4. Use the imported CA for issuance and certificate-chain retrieval.
5. If an ancestor must also sign, import its matching private key to enable
   signing on that existing CA.

Import validation, family assignment, and duplicate handling still need decisions
before this walkthrough defines a complete contract.

### 5.3 Rotate and stop issuance from an old CA

1. Create a new root in the same family, accepting the next root-series name.
2. Create replacement intermediates as needed; retain the old hierarchy.
3. Move issuance to the replacement CAs explicitly.
4. Disable new issuance on an old CA. Its existing certificates and chains
   remain retrievable, and its certificates are not automatically revoked.
5. Re-enable issuance if needed, subject to the usual issuance constraints.

### 5.4 Record a certificate revocation

1. Select an issued certificate and record its revocation reason and time.
2. Retain the certificate, issuer relationship, and revocation record.
3. Report that the revocation is recorded locally but is not published through
   CRLs or OCSP in v1. Relying parties have not been notified by this operation.

---

## 6. Consequences

### 6.1 Positive

- Multi-tier private PKI is a core open-source capability.
- Families retain a useful operator-facing identity across root changes.
- Suggested names reduce repetitive setup without forcing naming conventions.
- A shared HTTP API supports the initial CLI and a later web UI.
- A signing-capable key-provider boundary leaves room for future providers that
  retain private keys outside the service process.
- PEM import provides a route for adopting existing CA infrastructure.

### 6.2 Negative / trade-offs

- An unlocking secret introduces startup and recovery responsibilities for
  operators. Restarts interrupt signing until an operator unlocks the service;
  the detailed recovery procedure is still unresolved.
- Encrypted storage does not remove the need to protect keys while the software
  provider uses them in memory.
- Configurable providers do not by themselves make existing keys portable;
  changing providers requires a defined migration strategy.
- Importing a CA key and chain does not reconstruct its historical issuance or
  revocation records.
- Initial deployments record revocations but do not provide CRLs or OCSP
  responses; local revocation records alone do not enforce client rejection.
- Every authenticated caller has administrative power in v1; restricted
  application credentials require later authorisation support.
- Private-key export is unavailable in v1, so backup and recovery need their
  own procedure rather than relying on an export endpoint.

---

## 7. Deferred/open questions

### 7.1 Deferred capabilities and exclusions

Explicitly deferred: CRL publication, OCSP, ACME, the web UI, finer-grained
authorisation, and externally signed intermediate requests. Private-key export
is outside v1 scope.
Publicly trusted CA operation and tenant isolation are outside scope.

### 7.2 Security and recovery

- Encrypted backup/recovery policy.
- Unlock-secret delivery, explicit relocking behaviour, and key-provider migration.
- Token provisioning and lifecycle.

### 7.3 Import, naming, and hierarchy lifecycle

- Detailed PEM import validation, supported key encodings, ambiguous or incomplete
  chains, duplicate imports, and family assignment.
- CA retirement and deletion policy, and whether disabling an ancestor should
  affect issuance by its descendants.
- Defaults and ancestor enforcement for maximum path length.
- Naming-series allocation and collision behaviour, including custom names.

### 7.4 Issuance, retrieval, and revocation

- Issuance-profile defaults.
- Certificate and chain retrieval identifiers, formats, and access policy.
- Detailed revocation semantics, including CA revocation and effective-time
  handling, and how future CRL/OCSP publication URLs will be introduced.

These follow-up questions do not change the v1 scope above. Their answers must
be specified before implementing the affected behaviour.

---

## 8. References

- [RFC 5280: X.509 certificates and CRLs](https://www.rfc-editor.org/rfc/rfc5280.html),
  particularly section 4.2.1.9 on basic constraints and path length.
- [Envelope encryption](https://docs.cloud.google.com/kms/docs/envelope-encryption),
  a candidate pattern for the software-key provider; the exact construction is
  not yet selected.
