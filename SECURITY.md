# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in DemiDM, please report it responsibly.

### How to Report

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please report via:

1. **GitHub Security Advisories** (preferred): Go to the repository's Security tab and click "Report a vulnerability"
2. **Email**: Send details to the maintainers privately

### What to Include

- Description of the vulnerability
- Steps to reproduce or proof of concept
- Potential impact assessment
- Suggested fix (if any)

### Response Timeline

- **Initial acknowledgment**: Within 48 hours
- **Status update**: Within 7 days
- **Resolution target**: Within 30 days (depending on complexity)

### What to Expect

1. We will acknowledge receipt of your report
2. We will investigate and validate the issue
3. We will develop and test a fix
4. We will release a security patch
5. We will publicly disclose the issue after the patch is available

### Safe Harbor

We consider security research conducted in good faith to be protected under safe harbor principles. We will not pursue legal action against researchers who:

- Provide us reasonable time to fix the issue
- Avoid privacy violations and data destruction
- Do not disrupt service availability
- Only interact with accounts they own

## Security Best Practices for Contributors

When contributing to DemiDM, please follow these security guidelines:

### Password Handling

- Always use `secrecy::SecretString` for passwords
- Call `zeroize()` immediately after use
- Never log password content (even in debug mode)
- Never store passwords in memory longer than necessary

### Authentication

- PAM operations must run in isolated worker threads
- Validate all inputs before passing to PAM
- Handle all PAM errors without leaking information

### Session Management

- Drop privileges as soon as possible after authentication
- Use `setuid`/`setgid` correctly when launching sessions
- Validate session paths before execution

### Logging

- Never log sensitive information (passwords, tokens, keys)
- Use appropriate log levels (debug vs info vs error)
- Sanitize user input before including in logs

### Dependencies

- Keep dependencies up to date
- Run `cargo audit` regularly
- Review new dependencies carefully before adding

## Security Architecture

DemiDM follows these security principles:

1. **Least Privilege**: Run with minimal required permissions
2. **Defense in Depth**: Multiple layers of security checks
3. **Fail Secure**: Deny access on any error condition
4. **Zero Trust**: Validate all inputs, even from "trusted" sources
5. **Secure Defaults**: Safe configuration out of the box
