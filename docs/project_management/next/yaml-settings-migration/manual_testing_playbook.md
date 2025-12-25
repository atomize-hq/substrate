# YAML Settings Migration (Y0) — Manual Testing Playbook

This playbook validates the Y0 TOML→YAML runtime settings migration (`substrate config init/show/set`) end-to-end using a throwaway `HOME`.

Authoritative spec:
- `docs/project_management/next/yaml-settings-migration/Y0-spec.md`
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Automated smoke scripts

Run the platform smoke script first (it uses temp directories and exits non-zero on failure):
- Linux: `bash docs/project_management/next/yaml-settings-migration/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/next/yaml-settings-migration/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/next/yaml-settings-migration/smoke/windows-smoke.ps1`

## 0) Preconditions

1) Verify the CLI:
```bash
substrate --version
which substrate
```

2) Create an isolated test environment:
```bash
export Y0_TEST_HOME="$(mktemp -d)"
export Y0_TEST_WS="$(mktemp -d)"
echo "Y0_TEST_HOME=$Y0_TEST_HOME"
echo "Y0_TEST_WS=$Y0_TEST_WS"
```

## 1) YAML init creates `config.yaml` and does not create TOML

1) Run:
```bash
HOME="$Y0_TEST_HOME" substrate config init --force
echo "exit=$?"
```

Expected:
- Exit `0`.
- File exists: `$Y0_TEST_HOME/.substrate/config.yaml`
- File does not exist: `$Y0_TEST_HOME/.substrate/config.toml`

2) Verify files:
```bash
test -f "$Y0_TEST_HOME/.substrate/config.yaml"
echo "exit=$?"
test ! -e "$Y0_TEST_HOME/.substrate/config.toml"
echo "exit=$?"
```

Expected:
- Both commands exit `0`.

## 2) `config show` prints YAML and `--json` remains functional

1) Run:
```bash
HOME="$Y0_TEST_HOME" substrate config show
echo "exit=$?"
```

Expected:
- Exit `0`.
- Output contains at least `install:` and `world:`.

2) Run:
```bash
HOME="$Y0_TEST_HOME" substrate config show --json | jq -e '.world.anchor_mode' >/dev/null
echo "exit=$?"
```

Expected:
- Exit `0`.

## 3) `config set` updates YAML and persists

1) Run:
```bash
HOME="$Y0_TEST_HOME" substrate config set world.anchor_mode=follow-cwd
echo "exit=$?"
```

Expected:
- Exit `0`.

2) Verify persistence:
```bash
grep -q 'anchor_mode: follow-cwd' "$Y0_TEST_HOME/.substrate/config.yaml"
echo "exit=$?"
```

Expected:
- Exit `0`.

## 4) Directory settings override global config (precedence)

1) Create workspace settings:
```bash
cd "$Y0_TEST_WS"
mkdir -p .substrate
cat > .substrate/settings.yaml <<'YAML'
world:
  anchor_mode: project
  caged: true
YAML
```

2) Run from the workspace:
```bash
cd "$Y0_TEST_WS"
HOME="$Y0_TEST_HOME" substrate config show --json | jq -e '.world.anchor_mode=="project"' >/dev/null
echo "exit=$?"
```

Expected:
- Exit `0`.

## 5) TOML settings files are rejected with actionable error

1) Create an unsupported TOML file:
```bash
mkdir -p "$Y0_TEST_HOME/.substrate"
cat > "$Y0_TEST_HOME/.substrate/config.toml" <<'TOML'
[world]
anchor_mode = "project"
TOML
```

2) Run:
```bash
HOME="$Y0_TEST_HOME" substrate config show
echo "exit=$?"
```

Expected:
- Exit `2`.
- Error output mentions:
  - the TOML path (`config.toml`)
  - the YAML path (`config.yaml`)
  - the remediation steps from `Y0-spec.md` (delete TOML, run `substrate config init --force`, then `substrate config set ...`).

## 6) Cleanup

```bash
rm -rf "$Y0_TEST_HOME" "$Y0_TEST_WS"
unset Y0_TEST_HOME Y0_TEST_WS
```
