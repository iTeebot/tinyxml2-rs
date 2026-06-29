# Security Policy

Thank you for helping to keep **tinyxml2-rs** and its users safe. This document outlines the
security policy for the project, how to report vulnerabilities, and what to expect from the
maintainers.

> **tinyxml2-rs** is a ground-up Rust implementation of the TinyXML2 C++ API.
> Licensed under the MIT License — Copyright © 2026 Teebot.

---

## Supported Versions

The following table lists the versions of tinyxml2-rs that are currently receiving security
updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Yes              |
| < 0.1   | ❌ No               |

Only the latest patch release within a supported minor version is actively maintained. Users
are strongly encouraged to upgrade to the latest release.

---

## Reporting a Vulnerability

If you discover a security vulnerability in tinyxml2-rs, **please report it responsibly**.

📧 **Email:** [security-tinyxmls2-rs@iteebot.com](mailto:security-tinyxmls2-rs@iteebot.com)

> [!CAUTION]
> **Do NOT open a public GitHub issue for security vulnerabilities.** Public disclosure before
> a fix is available puts all users at risk.

### What to Include in Your Report

To help us triage and resolve the issue as quickly as possible, please include:

- **Version** — The exact version of tinyxml2-rs you are using (e.g., `0.1.3`).
- **Operating System** — Your OS and version (e.g., Ubuntu 24.04, macOS 15, Windows 11).
- **Description** — A clear and concise description of the vulnerability and its potential impact.
- **Reproduction Steps** — Detailed, step-by-step instructions to reproduce the issue, including
  any minimal XML input or code snippets required.

Providing complete and accurate information significantly speeds up our response.

---

## Response Timeline

We take every security report seriously and aim to respond promptly:

| Milestone               | Target Timeframe     |
| ------------------------ | -------------------- |
| **Acknowledgement**      | Within **48 hours**  |
| **Initial Assessment**   | Within **1 week**    |
| **Fix (Critical)**       | Within **30 days**   |

- You will receive an acknowledgement confirming receipt of your report within 48 hours.
- An initial assessment with severity classification will follow within one week.
- Critical vulnerabilities will be patched and released within 30 days of the initial report.
- Non-critical issues will be prioritized and addressed according to severity.

We will keep you informed of progress throughout the process.

---

## Scope

The following categories of security issues are **in scope** for this policy:

- **XML parsing vulnerabilities** — Incorrect parsing behavior that could lead to data
  corruption, information disclosure, or unexpected program state.
- **Memory safety issues** — Any violation of Rust's memory safety guarantees, including
  issues in `unsafe` blocks.
- **Denial of Service (DoS) via deeply nested XML** — Stack overflows or excessive resource
  consumption caused by pathologically deep XML document structures.
- **Entity expansion attacks** — Exponential entity expansion (e.g., "Billion Laughs" attack)
  or other entity-related resource exhaustion.
- **Buffer overflows** — Out-of-bounds reads or writes in any parsing or serialization path.
- **Panics on malformed input** — Unexpected panics or aborts when processing untrusted or
  malformed XML input that should be handled gracefully.

---

## Out of Scope

The following are **not** covered by this security policy:

- **Issues in third-party dependencies** — Vulnerabilities in upstream crates or libraries
  should be reported to the respective maintainers. If a dependency vulnerability affects
  tinyxml2-rs users, please let us know so we can evaluate and update accordingly.
- **Social engineering** — Phishing, credential theft, or other attacks targeting project
  maintainers or contributors rather than the software itself.

---

## Disclosure Policy

We follow a **coordinated disclosure** model:

1. **Report** — You privately report the vulnerability to [security-tinyxmls2-rs@iteebot.com](mailto:security-tinyxmls2-rs@iteebot.com).
2. **Triage & Fix** — We work with you to understand, reproduce, and fix the issue.
3. **Release** — A patched version is released and a security advisory is published.
4. **Public Disclosure** — Full details are made public after the fix is available.

> [!IMPORTANT]
> We request a **90-day disclosure window** from the date of your initial report. This gives
> us adequate time to develop, test, and release a fix before public disclosure.

If we are unable to address the issue within 90 days, we will coordinate with you on an
appropriate disclosure timeline.

---

## Recognition

We deeply appreciate the efforts of security researchers and community members who help
improve the security of tinyxml2-rs.

With your permission, we will acknowledge your contribution in:

- The **release notes** for the patched version.
- A dedicated **Security Acknowledgements** section in this repository.

If you prefer to remain anonymous, we will respect that preference completely.

Thank you for making tinyxml2-rs safer for everyone. 🛡️
