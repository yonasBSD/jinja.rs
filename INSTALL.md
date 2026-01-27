# Installation

## 🔍 How to Verify Releases

Every jinja.rs release includes multiple layers of integrity, authenticity, and provenance metadata.  
This section shows how to verify each part using standard open‑source tools.

### 1. Verify BLAKE3 Checksums

Each release includes a file named:

checksums-blake3.txt

To verify an artifact:
```console
    b3sum --check checksums-blake3.txt
```

If the output shows “OK”, the file matches the published checksum.

---

### 2. Verify Sigstore Signatures (Keyless)

All artifacts are signed using Sigstore Cosign with GitHub’s OIDC identity.  
Each artifact has a matching signature file:

`<artifact>.sig`

To verify:

```console
    cosign verify-blob \
      --signature <artifact>.sig \
      <artifact>
```

Cosign will confirm:
- the signature is valid
- it was created by GitHub Actions
- it matches the expected workflow identity

No keys or secrets are required.

---

### 3. Verify SLSA Level 3 Provenance

Each release includes a provenance attestation describing:
- the exact commit used
- the build environment
- the GitHub workflow identity
- the build steps
- the produced artifacts

To verify provenance:

```console
    cosign verify-attestation \
      --type slsaprovenance \
      --predicate-type slsaprovenance \
      <artifact>
```

This ensures the artifact was built by the trusted jinja.rs pipeline.

---

### 4. Inspect Per‑Artifact SBOMs

Each artifact includes its own Software Bill of Materials in SPDX JSON format:

`sbom-<artifact>.spdx.json`

To inspect an SBOM:

```console
    syft scan sbom-<artifact>.spdx.json
```

Or view it directly:

```console
    cat sbom-<artifact>.spdx.json
```

This provides full transparency into dependencies and build inputs.

---

### 5. Verify Attestations (SBOMs and Signatures)

Attestations are generated for:
- each SBOM
- each signature file

To verify an attestation:

```console
    cosign verify-attestation \
      --predicate-type https://slsa.dev/provenance/v1 \
      <subject-file>
```

This confirms the metadata itself is authentic and tamper‑proof.

---

### Summary

By combining:
- reproducible builds
- BLAKE3 integrity
- Sigstore keyless signatures
- per‑artifact SBOMs
- SLSA Level 3 provenance
- attestations for every critical file

jinja.rs provides a release pipeline with strong, modern supply‑chain guarantees that are rare even among large open‑source projects.
