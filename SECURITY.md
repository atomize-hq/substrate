# Security Policy

Security considerations and vulnerability reporting for Substrate.

## Security Model

### Current Security Features

**Command Tracing Security**:

- Log files created with 0600 permissions (user-only access)
- Comprehensive credential redaction for sensitive arguments
- SHA-256 binary fingerprinting for integrity verification
- Emergency bypass mechanism for recovery scenarios

**Path Security**:

- SHIM_ORIGINAL_PATH validation prevents directory inclusion attacks
- Binary resolution with permission verification
- Protection against PATH injection attacks

**Process Security**:

- Proper signal handling and process group management
- PTY security with terminal state restoration
- Session correlation prevents command confusion

### Future Security Architecture (Phase 4)

**World-Based Isolation**:

- Filesystem isolation via namespaces and overlayfs
- Network filtering via nftables/iptables
- Resource limits via cgroups v2
- Syscall filtering via seccomp

**Policy Enforcement**:

- YAML-based security policies with JSON schema validation
- Command allowlists and denylists with pattern matching
- Interactive approval workflows with diff previews
- Automatic policy reloading with atomic updates

**Agent Security**:

- Per-agent execution budgets and rate limiting
- Scope-based permission tokens via sealed file descriptors
- Audit logging for all agent interactions
- Resource usage tracking and enforcement

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |
| < 0.1.0 | No        |

## Reporting Vulnerabilities

### Security Issues

**DO NOT** create public GitHub issues for security vulnerabilities.

**Instead**, Uue GitHub's private vulnerability reporting

### What to Include

**Required Information**:

- Vulnerability description
- Steps to reproduce
- Impact assessment
- Affected versions
- Suggested mitigation

**Optional Information**:

- Proof of concept code
- Exploit scenarios
- Recommended fixes

### Response Timeline

- **Initial Response**: Within 48 hours
- **Assessment**: Within 7 days
- **Fix Development**: Varies by severity
- **Public Disclosure**: After fix is available

## Security Best Practices

### For Users

**Installation Security**:

- Build from source when possible
- Verify binary checksums
- Use official releases only
- Keep dependencies updated

**Runtime Security**:

- Review log files regularly
- Use credential redaction (avoid SHIM_LOG_OPTS=raw in production)
- Implement external log rotation
- Monitor for unexpected command patterns

**Environment Security**:

- Protect SHIM_ORIGINAL_PATH from untrusted directories
- Secure log file permissions (0600)
- Regular integrity verification of shim binaries
- Use emergency bypass only when necessary

### For Developers

**Code Security**:

- Never log sensitive information directly
- Use anyhow::Result for error handling
- Validate all external inputs
- Follow principle of least privilege

**Testing Security**:

- Test credential redaction patterns
- Verify file permission handling
- Test signal safety
- Validate error handling paths

## Known Security Limitations

### Current Limitations

- **Absolute Path Bypass**: Commands invoked with absolute paths skip shimming
- **Shell Builtin Limitation**: Built-ins in non-substrate shells aren't captured
- **Log Atomicity**: Large entries may interleave in high-concurrency scenarios
- **Host System Trust**: Relies on host system security for binary integrity

### Mitigation Strategies

- Use relative command names when possible
- Implement external log analysis for builtin detection
- Monitor for log integrity issues
- Harden host system security appropriately

### Future Mitigations

Phase 4 security worlds will address:

- Complete command interception via world isolation
- Comprehensive filesystem and network controls
- Reduced dependency on host system security
- Advanced audit capabilities with tamper detection

## Security Architecture

### Defense in Depth

**Layer 1**: Command interception and logging
**Layer 2**: Credential redaction and data protection
**Layer 3**: Binary integrity verification
**Layer 4**: Emergency bypass and recovery mechanisms

**Future Layers**:
**Layer 5**: World-based isolation and sandboxing
**Layer 6**: Policy-based access controls
**Layer 7**: Graph-based anomaly detection

### Threat Model

**In Scope**:

- Malicious command execution
- Credential exposure in logs
- Binary tampering
- PATH manipulation attacks

**Out of Scope**:

- Host system compromise
- Kernel vulnerabilities
- Hardware attacks
- Social engineering

## Compliance Considerations

### Data Handling

- **Personal Data**: Command lines may contain personal information
- **Credentials**: Automatic redaction with bypass option
- **Audit Trails**: Comprehensive logging for compliance requirements
- **Data Retention**: External log rotation and archival

### Regulatory Frameworks

Substrate supports compliance with:

- SOC 2 Type II (audit trail requirements)
- ISO 27001 (security management)
- GDPR (data protection, where applicable)
- Industry-specific requirements (depending on usage)

## Contact

For security-related questions or concerns:

- **General**: Create a GitHub discussion
- **Vulnerabilities**: Use private reporting channels
- **Compliance**: Contact maintainers for specific requirements
