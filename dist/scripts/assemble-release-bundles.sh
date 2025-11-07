#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: assemble-release-bundles.sh --tag <tag> --artifacts <dir> --output <dir>

  --tag        Release tag (e.g., v0.2.0)
  --artifacts  Path containing cargo-dist build artifacts downloaded from CI
  --output     Directory to write the final bundles + checksums into
USAGE
}

log() {
  printf '[bundle] %s\n' "$*"
}

fatal() {
  printf '[bundle][ERROR] %s\n' "$*" >&2
  exit 1
}

TAG=""
ARTIFACTS_DIR=""
OUTPUT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      TAG="$2"
      shift 2
      ;;
    --artifacts)
      ARTIFACTS_DIR="$2"
      shift 2
      ;;
    --output)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      fatal "Unknown argument: $1"
      ;;
  esac
done

[[ -n "$TAG" ]] || fatal "--tag is required"
[[ -n "$ARTIFACTS_DIR" ]] || fatal "--artifacts is required"
[[ -n "$OUTPUT_DIR" ]] || fatal "--output is required"

VERSION="${TAG#v}"
[[ -n "$VERSION" ]] || fatal "Tag $TAG must start with 'v' followed by the version number"

ARTIFACTS_DIR="$(realpath "$ARTIFACTS_DIR")"
OUTPUT_DIR="$(realpath -m "$OUTPUT_DIR")"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$OUTPUT_DIR"
rm -rf "$OUTPUT_DIR"/*

WORK_DIR="$TMP_DIR/work"
SUPPORT_DIR="$TMP_DIR/support"
STAGING_DIR="$TMP_DIR/staging"
mkdir -p "$WORK_DIR" "$SUPPORT_DIR" "$STAGING_DIR"

# Mapping between bundle labels and Rust targets we build.
declare -A HOST_TARGETS=(
  [linux_x86_64]="x86_64-unknown-linux-gnu"
  [linux_aarch64]="aarch64-unknown-linux-gnu"
  [macos_x86_64]="x86_64-apple-darwin"
  [macos_arm64]="aarch64-apple-darwin"
  [windows_x86_64]="x86_64-pc-windows-msvc"
)

declare -A HOST_KIND=(
  [linux_x86_64]="linux"
  [linux_aarch64]="linux"
  [macos_x86_64]="mac"
  [macos_arm64]="mac"
  [windows_x86_64]="windows"
)

declare -A GUEST_AGENT_TARGET=(
  [linux_x86_64]="x86_64-unknown-linux-gnu"
  [linux_aarch64]="aarch64-unknown-linux-gnu"
  [macos_x86_64]="x86_64-unknown-linux-gnu"
  [macos_arm64]="aarch64-unknown-linux-gnu"
  [windows_x86_64]="x86_64-unknown-linux-gnu"
)

declare -A ARCHIVE_EXT=(
  [windows_x86_64]="zip"
  [linux_x86_64]="tar.gz"
  [linux_aarch64]="tar.gz"
  [macos_x86_64]="tar.gz"
  [macos_arm64]="tar.gz"
)

BUNDLE_LABELS=(
  linux_x86_64
  linux_aarch64
  macos_x86_64
  macos_arm64
  windows_x86_64
)

find_artifact() {
  local crate="$1"
  local target="$2"
  local matches=()
  while IFS= read -r -d '' path; do
    matches+=("$path")
  done < <(find "$ARTIFACTS_DIR" -type f \( -name "${crate}-${target}.tar.xz" -o -name "${crate}-${target}.tar.gz" -o -name "${crate}-${target}.zip" \) -print0)

  [[ ${#matches[@]} -gt 0 ]] || fatal "Could not locate artifact for ${crate}-${target} in $ARTIFACTS_DIR"
  if [[ ${#matches[@]} -gt 1 ]]; then
    fatal "Multiple artifacts found for ${crate}-${target}; prune $ARTIFACTS_DIR to be unambiguous"
  fi
  printf '%s\n' "${matches[0]}"
}

extract_archive() {
  local archive="$1"
  local dest="$2"
  mkdir -p "$dest"
  case "$archive" in
    *.zip)
      unzip -q "$archive" -d "$dest"
      ;;
    *)
      tar -xf "$archive" -C "$dest"
      ;;
  esac
}

find_extracted_root() {
  local dir="$1"
  local entries=()
  while IFS= read -r -d '' entry; do
    entries+=("$entry")
  done < <(find "$dir" -mindepth 1 -maxdepth 1 -print0)

  if [[ ${#entries[@]} -eq 1 && -d "${entries[0]}" ]]; then
    printf '%s\n' "${entries[0]}"
  else
    printf '%s\n' "$dir"
  fi
}

copy_bin_from_artifact() {
  local crate="$1"
  local target="$2"
  local dest="$3"

  local artifact
  artifact="$(find_artifact "$crate" "$target")"
  local extract_dir="$WORK_DIR/${crate}-${target}"
  rm -rf "$extract_dir"
  mkdir -p "$extract_dir"
  extract_archive "$artifact" "$extract_dir"
  local root
  root="$(find_extracted_root "$extract_dir")"
  local bin_dir="$root/bin"
  if [[ ! -d "$bin_dir" ]]; then
    bin_dir="$root"
  fi

  if [[ ! -d "$bin_dir" ]]; then
    fatal "Archive ${artifact} does not contain expected binaries"
  fi

  mkdir -p "$dest"
  shopt -s nullglob
  for file in "$bin_dir"/*; do
    if [[ -f "$file" ]]; then
      local name
      name="$(basename "$file")"
      case "$name" in
        README*|LICENSE*|CHANGELOG*|*.md)
          continue
          ;;
      esac
      cp "$file" "$dest/$name"
      chmod +x "$dest/$name" 2>/dev/null || true
    fi
  done
  shopt -u nullglob
}

stage_support_assets() {
  local support_tar support_zip
  support_tar="$(find "$ARTIFACTS_DIR" -type f -name 'substrate-support.tar.gz' | head -n1)"
  support_zip="$(find "$ARTIFACTS_DIR" -type f -name 'substrate-support.zip' | head -n1)"
  [[ -n "$support_tar" ]] || fatal "substrate-support.tar.gz not found in $ARTIFACTS_DIR"
  [[ -n "$support_zip" ]] || fatal "substrate-support.zip not found in $ARTIFACTS_DIR"

  tar -xzf "$support_tar" -C "$SUPPORT_DIR"

  cp "$support_tar" "$OUTPUT_DIR/"
  cp "$support_zip" "$OUTPUT_DIR/"

  SUPPORT_TAR_OUT="$OUTPUT_DIR/$(basename "$support_tar")"
  SUPPORT_ZIP_OUT="$OUTPUT_DIR/$(basename "$support_zip")"
}

copy_support_contents() {
  local bundle_root="$1"
  if [[ -d "$SUPPORT_DIR/docs" ]]; then
    mkdir -p "$bundle_root/docs"
    cp -a "$SUPPORT_DIR/docs/." "$bundle_root/docs/"
  fi
  if [[ -d "$SUPPORT_DIR/scripts" ]]; then
    mkdir -p "$bundle_root/scripts"
    cp -a "$SUPPORT_DIR/scripts/." "$bundle_root/scripts/"
  fi
}

package_bundle() {
  local label="$1"
  local target="$2"
  local kind="$3"
  local guest_agent="$4"

  local bundle_dir="$STAGING_DIR/$label"
  rm -rf "$bundle_dir"
  mkdir -p "$bundle_dir/bin"

  copy_support_contents "$bundle_dir"
  copy_bin_from_artifact "substrate" "$target" "$bundle_dir/bin"
  copy_bin_from_artifact "host-proxy" "$target" "$bundle_dir/bin"

  if [[ "$label" == "windows_x86_64" ]]; then
    copy_bin_from_artifact "substrate-forwarder" "$target" "$bundle_dir/bin"
  fi

  if [[ "$kind" == "linux" ]]; then
    copy_bin_from_artifact "world-agent" "$target" "$bundle_dir/bin"
  else
    local guest_dir="$bundle_dir/bin/linux"
    mkdir -p "$guest_dir"
    copy_bin_from_artifact "world-agent" "$guest_agent" "$guest_dir"
  fi

  local archive_name="substrate-v${VERSION}-${label}."
  archive_name+="${ARCHIVE_EXT[$label]}"
  local archive_path="$OUTPUT_DIR/$archive_name"

  if [[ "$kind" == "windows" ]]; then
    (cd "$STAGING_DIR" && zip -qr "$archive_path" "$label")
  else
    (cd "$STAGING_DIR" && tar -czf "$archive_path" "$label")
  fi
  GENERATED_FILES+=("$archive_path")
  log "Created bundle $archive_name"
}

stage_support_assets
GENERATED_FILES=()
GENERATED_FILES+=("$SUPPORT_TAR_OUT" "$SUPPORT_ZIP_OUT")

for label in "${BUNDLE_LABELS[@]}"; do
  package_bundle "$label" "${HOST_TARGETS[$label]}" "${HOST_KIND[$label]}" "${GUEST_AGENT_TARGET[$label]}"
done

sha256sum "${GENERATED_FILES[@]}" > "$OUTPUT_DIR/SHA256SUMS"
log "Wrote SHA256SUMS"
