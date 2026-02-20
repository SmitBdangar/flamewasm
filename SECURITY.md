# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| 0.x (main) | ✅ |

## Reporting a Vulnerability

**Please do NOT open a public GitHub issue for security vulnerabilities.**

Email: `security@flamewasm.dev` (or open a [GitHub private security advisory](https://github.com/SmitBdangar/flamewasm/security/advisories/new)).

Include:
- A description of the vulnerability
- Steps to reproduce (minimal PoC wasm/wat file if applicable)
- Potential impact
- Suggested fix (optional)

We aim to respond within **72 hours** and patch within **14 days** for critical issues.

## Scope

The following are in scope:

- **Sandbox escapes**: A Wasm module bypassing its capability policy
- **Memory safety**: Out-of-bounds access leaking host memory
- **Parser panics**: Crafted WASM crashing the host process (instead of returning an error)
- **WASI leaks**: A denied WASI path being accessible despite the policy

The following are **out of scope**:

- DoS via excessive resource consumption (tracked as regular issues)
- Issues requiring pre-existing host compromise
