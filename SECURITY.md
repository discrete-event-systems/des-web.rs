# Security policy

Please report suspected vulnerabilities privately through GitHub's
**Security → Report a vulnerability** flow for this repository. Do not open a
public issue with exploit details, credentials, customer data, or database
connection details.

Supported code is the latest commit on `main`.

This service must keep secrets in the runtime environment or GitHub Actions
secrets. Never commit Supabase keys, database URLs, the private pg-defs deploy
key, or upstream authentication values. The GitHub Actions key for the
`libs/` submodule must remain read-only and scoped only to
`ORESoftware/k8s-libs-and-shared-defs`.
