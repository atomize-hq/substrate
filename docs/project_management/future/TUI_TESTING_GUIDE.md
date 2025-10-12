# Comprehensive TUI Testing Guide for Substrate

## Executive Summary

This document provides testing instructions for 145+ Terminal User Interface (TUI) applications that should be tested with Substrate's PTY detection. The current `KNOWN_TUIS` list covers only ~23% of common TUIs, leaving significant gaps in coverage.

## Key Findings

- **Current Coverage**: 34 TUIs in KNOWN_TUIS
- **Missing Coverage**: 111+ TUIs need to be added
- **Total Applications**: 145+ TUIs across 11 categories
- **Critical Gaps**: Package managers, debuggers, database CLIs

## Testing Strategy

### Container-First Approach (Recommended)

Instead of installing 100+ packages on a host machine, use Docker containers for isolated testing:

```bash
# Example Dockerfile.debian
FROM debian:bullseye
RUN apt-get update && apt-get install -y \
    curl wget git build-essential python3-pip golang cargo \
    && rm -rf /var/lib/apt/lists/*
COPY test_scripts/ /app/
WORKDIR /app
```

```bash
# Build and run tests
docker build -t tui-test:debian -f Dockerfile.debian .
docker run --rm -it tui-test:debian bash run_tests.sh
```

### Test Success Criteria

A test is considered successful if:
1. The TUI launches without crashing in Substrate's PTY
2. The application runs for the timeout duration OR exits cleanly
3. Exit code is 0 (clean exit) or 124 (timeout)

**Note**: This tests PTY compatibility, not visual rendering or functionality.

## Priority Additions (Top 20)

These should be added to KNOWN_TUIS immediately:

1. **micro** - Modern terminal editor
2. **mc** - Midnight Commander file manager
3. **gdb** - GNU Debugger
4. **lldb** - LLVM Debugger
5. **mycli** - MySQL CLI with auto-completion
6. **pgcli** - PostgreSQL CLI with auto-completion
7. **redis-cli** - Redis CLI
8. **iotop** - I/O usage monitor (requires root)
9. **iftop** - Network bandwidth monitor (requires root)
10. **nethogs** - Network usage by process
11. **helix** (hx) - Post-modern modal editor
12. **lf** - Terminal file manager
13. **nnn** - Fast terminal file browser
14. **lazydocker** - Docker TUI
15. **irssi** - IRC client
16. **weechat** - IRC/chat client
17. **lynx** - Text web browser
18. **mosh** - Mobile shell
19. **vifm** - Vi-like file manager
20. **bottom** (btm) - System monitor

## Testing by Category

### Text Editors (Priority 1)

#### micro
```bash
# Install
Ubuntu/Debian: sudo apt install micro
macOS: brew install micro
Direct: curl https://getmic.ro | bash

# Prerequisites: None

# Test
substrate -c "micro test.txt"

# Cleanup
Ubuntu/Debian: sudo apt remove micro
macOS: brew uninstall micro
```

#### helix (hx)
```bash
# Install
Ubuntu/Debian: sudo snap install helix --classic
macOS: brew install helix
Cargo: cargo install helix-editor

# Prerequisites: None

# Test
substrate -c "hx test.txt"

# Cleanup
Ubuntu/Debian: sudo snap remove helix
macOS: brew uninstall helix
```

### System Monitors (Priority 2)

#### iotop
```bash
# Install
Ubuntu/Debian: sudo apt install iotop
macOS: brew install iotop
RHEL/CentOS: sudo yum install iotop

# Prerequisites: Root access

# Test
substrate -c "sudo iotop"

# Cleanup
Ubuntu/Debian: sudo apt remove iotop
macOS: brew uninstall iotop
```

### File Managers (Priority 3)

#### mc (Midnight Commander)
```bash
# Install
Ubuntu/Debian: sudo apt install mc
macOS: brew install midnight-commander
RHEL/CentOS: sudo yum install mc

# Prerequisites: None

# Test
substrate -c "mc"

# Cleanup
Ubuntu/Debian: sudo apt remove mc
macOS: brew uninstall midnight-commander
```

### Database CLIs (Priority 4)

#### mycli
```bash
# Install
All platforms: pip install mycli
Ubuntu/Debian: sudo apt install mycli

# Prerequisites: MySQL/MariaDB server running

# Test
substrate -c "mycli --help"  # Non-interactive
substrate -c "mycli -h localhost -u test"  # Interactive (if MySQL available)

# Cleanup
pip uninstall mycli
```

### Development Tools (Priority 5)

#### gdb
```bash
# Install
Ubuntu/Debian: sudo apt install gdb
macOS: brew install gdb
RHEL/CentOS: sudo yum install gdb

# Prerequisites: None

# Test
substrate -c "gdb"

# Cleanup
Ubuntu/Debian: sudo apt remove gdb
macOS: brew uninstall gdb
```

## Automated Testing Script

```bash
#!/bin/bash
# comprehensive_tui_test.sh

# Enhanced test function with prerequisites
test_tui() {
    local tui_command=$1
    local install_spec=$2
    local prereq_check=${3:-"true"}
    
    echo "--- Testing: $tui_command ---"
    
    # Check prerequisites
    if ! eval "$prereq_check"; then
        echo "SKIPPED (Prerequisites not met)"
        return
    fi
    
    # Check if installed
    if ! command -v "${tui_command%% *}" &> /dev/null; then
        echo "Installing... ($install_spec)"
        local install_type=$(echo "$install_spec" | cut -d: -f1)
        local install_package=$(echo "$install_spec" | cut -d: -f2-)
        
        case "$install_type" in
            apt) sudo apt-get install -y "$install_package" ;;
            brew) brew install "$install_package" ;;
            pip) pip3 install "$install_package" ;;
            go) go install "$install_package" ;;
            cargo) cargo install "$install_package" ;;
            snap) sudo snap install "$install_package" ;;
            *) echo "ERROR: Unknown installer"; return 1 ;;
        esac
    fi
    
    # Run test with substrate
    timeout 3s substrate -c "$tui_command"
    local exit_code=$?
    
    # Evaluate result
    if [[ $exit_code -eq 124 || $exit_code -eq 0 ]]; then
        echo "✓ SUCCESS"
    else
        echo "✗ FAILURE (Exit code: $exit_code)"
    fi
}

# Test examples with prerequisites
test_tui "micro test.txt" "apt:micro"
test_tui "mc" "apt:mc"
test_tui "gdb" "apt:gdb"
test_tui "iotop" "apt:iotop" "[ $EUID -eq 0 ]"
test_tui "mycli" "pip:mycli" "mysqladmin ping 2>/dev/null"
test_tui "lazydocker" "go:github.com/jesseduffield/lazydocker@latest" "docker info 2>/dev/null"
test_tui "k9s" "snap:k9s" "[ -f ~/.kube/config ]"
```

## Complete Testing Matrix

| Category | Total | In KNOWN_TUIS | To Add | Priority |
|----------|-------|---------------|---------|----------|
| Text Editors | 19 | 6 | 13 | High |
| System Monitors | 16 | 4 | 12 | High |
| File Managers | 12 | 2 | 10 | High |
| Database CLIs | 12 | 3 | 9 | High |
| Dev Tools | 15 | 7 | 8 | High |
| Package Managers | 12 | 0 | 12 | Medium |
| Network Tools | 18 | 3 | 15 | Medium |
| Container/K8s | 7 | 1 | 6 | Medium |
| REPLs | 22+ | 2 | 20+ | Low |
| Multiplexers | 5 | 3 | 2 | Low |
| Misc Tools | 7+ | 3 | 4+ | Low |

## Prerequisites by Tool Type

### Standalone Tools
- Text editors, file managers, system monitors (most)
- No external dependencies

### Service-Dependent Tools
- **Database CLIs**: Require running database server
- **Container Tools**: Require Docker daemon
- **K8s Tools**: Require valid kubeconfig
- **Network Monitors**: Often require root access

### Platform-Specific Considerations
- **Linux**: Most tools available via package managers
- **macOS**: Primarily use Homebrew; some GNU tools have different names
- **Windows**: Limited TUI support; consider WSL for testing

## CI/CD Integration

```yaml
# Example GitHub Actions workflow
name: TUI Compatibility Tests
on: [push, pull_request]

jobs:
  test-tuis:
    runs-on: ubuntu-latest
    container:
      image: tui-test:debian
    steps:
      - uses: actions/checkout@v3
      - name: Run TUI Tests
        run: |
          cd /app
          ./comprehensive_tui_test.sh
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: tui-test-results
          path: test-results.log
```

## Maintenance Recommendations

1. **Quarterly Reviews**: Review and update TUI list quarterly
2. **Community Input**: Accept PRs for new TUI additions
3. **Version Testing**: Test major version updates of critical TUIs
4. **Pattern Recognition**: Consider regex patterns for common TUI types
5. **Documentation**: Keep this guide updated with findings

## Appendix: Full TUI List by Category

### Text Editors
- **In KNOWN_TUIS**: vim, vi, nvim, neovim, nano, emacs
- **To Add**: micro, helix (hx), kakoune, joe, jed, pico, ne, tilde, vis, nvi, ex, ed, mcedit

### System Monitors  
- **In KNOWN_TUIS**: top, htop, btop, glances
- **To Add**: iotop, iftop, nethogs, nmon, bottom (btm), ytop, gotop, gtop, bpytop, bashtop, atop, s-tui

### File Managers
- **In KNOWN_TUIS**: ranger, yazi
- **To Add**: mc, vifm, lf, nnn, broot, fff, clifm, joshuto, xplr, hunter

### Database CLIs
- **In KNOWN_TUIS**: sqlite3, psql, mysql
- **To Add**: mycli, pgcli, litecli, mssql-cli, iredis, mongo, mongosh, redis-cli, usql

### Development Tools
- **In KNOWN_TUIS**: fzf, lazygit, gitui, tig, k9s, ipython, bpython
- **To Add**: gdb, lldb, radare2, lazydocker, dive, grv, ptpython, pudb

### Package Managers
- **In KNOWN_TUIS**: None
- **To Add**: apt, yum, dnf, pacman, npm, yarn, pnpm, cargo, pip, gem, composer, brew

### Network Tools
- **In KNOWN_TUIS**: telnet, ftp, sftp
- **To Add**: irssi, weechat, lynx, links, w3m, elinks, mosh, ncftp, lftp, mutt, neomutt, alpine

### Container/K8s Tools
- **In KNOWN_TUIS**: k9s
- **To Add**: lazydocker, dive, kdash, kubectl (exec -it), sen, dockly

### REPLs/Interpreters
- **In KNOWN_TUIS**: ipython, bpython
- **To Add**: irb, pry, iex, erl, ghci, racket, guile, sbcl, lua, R, julia, scala, jshell

### Terminal Multiplexers
- **In KNOWN_TUIS**: tmux, screen, zellij
- **To Add**: byobu, dvtm

### Miscellaneous
- **In KNOWN_TUIS**: nmtui, claude, codex, gemini, atomize
- **To Add**: alsamixer, dialog, whiptail, cmatrix, tty-clock