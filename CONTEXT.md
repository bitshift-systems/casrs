# Private PKI

A reusable open-source service for operating private certificate authorities and managing certificates.

## Language

**CA family**:
A named, long-lived grouping of related certificate authorities whose identity persists across root rotation. A family can provide serial naming suggestions for its root and intermediate CAs while permitting custom names.

**CA naming series**:
One of a family's two independent naming sequences: roots (`R1`, `R2`, …) or intermediates (`I1`, `I2`, …). Both sequences span the family, so intermediate numbering does not restart when its root changes; a suggested name is not the CA's identity.

**Root CA**:
A certificate authority at the top of a CA hierarchy.

**Intermediate CA**:
A certificate authority certified by another CA in the hierarchy, which may itself be a root or an intermediate CA.

**Certificate-only CA**:
A CA whose certificate is known but whose private key is not available for signing. It can represent an ancestor in an imported chain without being an operational issuer.

**Issuance profile**:
A named policy defining permitted certificate identities, usages, and maximum lifetime.

**Issuance disabled**:
A reversible CA state that prevents new issuance while retaining existing certificates and chains. It does not mean that existing certificates have been revoked.

**Revocation record**:
A record that a certificate has been revoked, including its reason and time. Recording revocation is distinct from publishing that status to relying parties.
