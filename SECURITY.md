# Security policy

## Supported versions

| Version | Supported |
|---|---|
| 0.1.x | Yes |

## Reporting a vulnerability

Use GitHub's private vulnerability reporting:

<https://github.com/spacelibres/STKCraft-time/security/advisories/new>

Do not open a public issue for a vulnerability. Include:

- the affected version and enabled features;
- a minimal reproduction;
- expected and actual behaviour;
- an impact assessment.

We aim to acknowledge reports within 7 days and to publish a fix and an
advisory for confirmed issues.

## Scope

`stkcraft-time` performs no network I/O. The main security boundary is
parsing untrusted LSK, SCLK and EOP text, and the local file paths passed
to the optional MCP server. The crate forbids unsafe code.
