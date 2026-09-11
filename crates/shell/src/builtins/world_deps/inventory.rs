use crate::execution::config_model;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostPlatform {
    Linux,
    MacOs,
    Windows,
}

impl HostPlatform {
    pub(crate) fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    fn matches(self, platforms: &[HostPlatform]) -> bool {
        platforms.contains(&self)
    }
}

impl std::fmt::Display for HostPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HostPlatform::Linux => "linux",
            HostPlatform::MacOs => "macos",
            HostPlatform::Windows => "windows",
        })
    }
}

fn parse_platform(raw: &str) -> Option<HostPlatform> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "linux" => Some(HostPlatform::Linux),
        "macos" => Some(HostPlatform::MacOs),
        "windows" => Some(HostPlatform::Windows),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InstallMethodV1 {
    Apt,
    Pacman,
    Script,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AptSpecV1 {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallDefV1 {
    pub method: InstallMethodV1,
    #[serde(default)]
    pub apt: Vec<AptSpecV1>,
    #[serde(default)]
    pub pacman: Vec<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub script_path: Option<String>,
    #[serde(default)]
    pub manual_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProbeDefV1 {
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum WrapperKindV1 {
    BashFunction(BashFunctionWrapperDefV1),
    BashSourceExec(BashSourceExecWrapperDefV1),
    ShEnvExec(ShEnvExecWrapperDefV1),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BashFunctionWrapperDefV1 {
    pub bash_source: String,
    pub function: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BashSourceExecWrapperDefV1 {
    pub bash_source: String,
    pub exec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShEnvExecWrapperDefV1 {
    pub exec: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct WrapperDefV1 {
    pub name: String,
    #[serde(flatten)]
    pub kind: WrapperKindV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PackageDefV1 {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub runnable: bool,
    #[serde(default)]
    pub entrypoints: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub wrappers: Vec<WrapperDefV1>,
    pub install: InstallDefV1,
    #[serde(default)]
    pub probe: Option<ProbeDefV1>,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub(crate) definition_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleDefV1 {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum InventoryItemDefV1 {
    // Box to avoid a large enum (clippy::large_enum_variant).
    Package(Box<PackageDefV1>),
    Bundle(BundleDefV1),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct InventoryViewV1 {
    pub packages: BTreeMap<String, PackageDefV1>,
    pub bundles: BTreeMap<String, BundleDefV1>,
}

impl InventoryViewV1 {
    pub(crate) fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.bundles.is_empty()
    }

    pub(crate) fn get(&self, name: &str) -> Option<InventoryItemDefV1> {
        if let Some(pkg) = self.packages.get(name) {
            return Some(InventoryItemDefV1::Package(Box::new(pkg.clone())));
        }
        if let Some(bundle) = self.bundles.get(name) {
            return Some(InventoryItemDefV1::Bundle(bundle.clone()));
        }
        None
    }

    pub(crate) fn validate_no_collisions(&self) -> Result<()> {
        for name in self.packages.keys() {
            if self.bundles.contains_key(name) {
                return Err(config_model::user_error(format!(
                    "invalid deps inventory: name collision: '{name}' exists in both packages and bundles"
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InventoryListItemSummaryV1 {
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runnable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<InstallMethodV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entrypoints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub(crate) fn builtin_inventory_v1(platform: HostPlatform) -> InventoryViewV1 {
    let mut view = InventoryViewV1::default();
    for pkg in builtin_packages_v1() {
        if is_visible_on_platform(&pkg.platforms, platform).unwrap_or(true) {
            view.packages.insert(pkg.name.clone(), pkg);
        }
    }
    for bundle in builtin_bundles_v1() {
        if is_visible_on_platform(&bundle.platforms, platform).unwrap_or(true) {
            view.bundles.insert(bundle.name.clone(), bundle);
        }
    }
    view
}

pub(crate) fn codex_runtime_install_script_template_v1() -> &'static str {
    r#"#!/usr/bin/env bash
set -euo pipefail

# Verified Packet 2 runtime posture: the official Codex musl release is self-contained
# in the guest for `codex --version`, so this package installs only the Codex binary and
# does not widen into a Node/npm runtime bundle.
world_deps_root="/var/lib/substrate/world-deps"
world_deps_bin="${world_deps_root}/bin"
package_root="${world_deps_root}/__SUBSTRATE_CODEX_PACKAGE_NAME__"
package_bin="${package_root}/bin"
downloads_root="${package_root}/downloads"
installed_binary="${package_bin}/codex"
archive_name="__SUBSTRATE_CODEX_ARCHIVE_NAME__"
archive_path="${downloads_root}/${archive_name}"
archive_url="__SUBSTRATE_CODEX_ARCHIVE_URL__"
archive_sha256="__SUBSTRATE_CODEX_ARCHIVE_SHA256__"
archive_entry_path="__SUBSTRATE_CODEX_ARCHIVE_ENTRY_PATH__"
executable_sha256="__SUBSTRATE_CODEX_EXECUTABLE_SHA256__"
codex_version="__SUBSTRATE_CODEX_VERSION__"
target_triple="__SUBSTRATE_CODEX_TARGET_TRIPLE__"

run_download_with_timeout() {
  local max_seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "${max_seconds}" "$@"
  else
    "$@"
  fi
}

mkdir -p "${world_deps_bin}" "${package_bin}" "${downloads_root}"

if [ ! -f "${archive_path}" ] || ! echo "${archive_sha256}  ${archive_path}" | sha256sum -c - >/dev/null 2>&1; then
  tmp_archive="${archive_path}.tmp"
  rm -f "${tmp_archive}"
  if [[ "${archive_url}" == file://* ]]; then
    echo "substrate: staging Codex runtime ${codex_version} for ${target_triple} from ${archive_url}" >&2
    cp "${archive_url#file://}" "${tmp_archive}"
  elif command -v wget >/dev/null 2>&1; then
    echo "substrate: downloading Codex runtime ${codex_version} for ${target_triple} from ${archive_url} with wget" >&2
    run_download_with_timeout 900 wget --tries=4 --timeout=15 --waitretry=5 --output-document "${tmp_archive}" "${archive_url}"
  elif command -v python3 >/dev/null 2>&1; then
    echo "substrate: downloading Codex runtime ${codex_version} for ${target_triple} from ${archive_url} with python3" >&2
    run_download_with_timeout 900 python3 - "${archive_url}" "${tmp_archive}" <<'PY'
import shutil
import socket
import sys
import urllib.request

url, destination = sys.argv[1], sys.argv[2]
socket.setdefaulttimeout(15)
with urllib.request.urlopen(url, timeout=15) as response, open(destination, "wb") as out:
    shutil.copyfileobj(response, out)
PY
  else
    echo "substrate: downloading Codex runtime ${codex_version} for ${target_triple} from ${archive_url} with curl" >&2
    curl -fsSL --retry 3 --connect-timeout 15 --speed-time 30 --speed-limit 1024 --max-time 900 --location "${archive_url}" -o "${tmp_archive}"
  fi
  echo "${archive_sha256}  ${tmp_archive}" | sha256sum -c -
  mv "${tmp_archive}" "${archive_path}"
fi

stage_dir="$(mktemp -d "${package_root}/.stage.${target_triple}.XXXXXX")"
trap 'rm -rf "${stage_dir}"' EXIT
mapfile -t archive_entries < <(tar -tzf "${archive_path}")
if [ "${#archive_entries[@]}" -ne 1 ] || [ "${archive_entries[0]}" != "${archive_entry_path}" ]; then
  echo "substrate: world deps package '__SUBSTRATE_CODEX_PACKAGE_NAME__' did not contain the one pinned Codex archive entry ${archive_entry_path}" >&2
  exit 1
fi
tar --no-same-owner --no-same-permissions -xzf "${archive_path}" -C "${stage_dir}" -- "${archive_entry_path}"
resolved_binary="${stage_dir}/${archive_entry_path}"
if [ ! -f "${resolved_binary}" ] || [ -L "${resolved_binary}" ]; then
  echo "substrate: pinned Codex archive entry is not one regular file" >&2
  exit 1
fi
echo "${executable_sha256}  ${resolved_binary}" | sha256sum -c -

echo "substrate: installing Codex runtime ${codex_version} for ${target_triple}" >&2
install_tmp="${package_bin}/.codex-install.$$.tmp"
install -m 0755 "${resolved_binary}" "${install_tmp}"
sync "${install_tmp}"
mv -f "${install_tmp}" "${installed_binary}"
sync "${package_bin}"
echo "${executable_sha256}  ${installed_binary}" | sha256sum -c -
ln -sf "${installed_binary}" "${world_deps_bin}/codex"
__SUBSTRATE_CODEX_INSTALLER_ARTIFACT_SOURCE_V1__
"#
}

pub(crate) fn render_codex_installer_artifact_source_v1() -> &'static str {
    r#"artifact_source_root="${world_deps_root}/runtime-artifacts-v1"
install -d -m 0750 "${artifact_source_root}" "${artifact_source_root}/records" "${artifact_source_root}/records/codex-0.125.0-x86_64-unknown-linux-musl" "${artifact_source_root}/heads"
touch "${artifact_source_root}/lock"
chmod 0640 "${artifact_source_root}/lock"
exec 9<>"${artifact_source_root}/lock"
flock -x 9
python3 - "${artifact_source_root}" "${installed_binary}" "${archive_name}" "${archive_url}" "${archive_sha256}" "${archive_entry_path}" "${executable_sha256}" <<'PY'
import datetime
import grp
import hashlib
import json
import os
import re
import secrets
import stat
import struct
import sys
import time
import uuid

root, executable, archive_name, archive_url, archive_sha256, archive_entry, executable_sha256 = sys.argv[1:]
stream_name = "codex-0.125.0-x86_64-unknown-linux-musl"

def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()

def domain_hash(domain, member, value, omitted):
    copied = dict(value)
    copied.pop(omitted)
    return hashlib.sha256(canonical({"domain": domain, member: copied})).hexdigest()

def timestamp():
    now = datetime.datetime.now(datetime.timezone.utc)
    return now.strftime("%Y-%m-%dT%H:%M:%S.%fZ")

def uuid7():
    milliseconds = time.time_ns() // 1_000_000
    random_bits = secrets.randbits(74)
    value = ((milliseconds & ((1 << 48) - 1)) << 80)
    value |= 7 << 76
    value |= ((random_bits >> 62) & 0xFFF) << 64
    value |= 2 << 62
    value |= random_bits & ((1 << 62) - 1)
    return str(uuid.UUID(int=value))

def valid_id(value, prefix):
    if not isinstance(value, str) or not value.startswith(prefix):
        return False
    try:
        parsed = uuid.UUID(value[len(prefix):])
    except (ValueError, AttributeError):
        return False
    return parsed.version == 7 and str(parsed) == value[len(prefix):]

def valid_digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None

def valid_u32(value):
    return type(value) is int and 0 <= value <= 0xffffffff

def valid_u64(value):
    return type(value) is int and 0 <= value <= 0xffffffffffffffff

def valid_timestamp(value):
    if not isinstance(value, str) or re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{6}Z", value) is None:
        return False
    try:
        datetime.datetime.strptime(value, "%Y-%m-%dT%H:%M:%S.%fZ")
    except ValueError:
        return False
    return True

def validate_store_value(value):
    expected = {"schema_version", "source_store_id", "root", "created_at", "store_hash"}
    identity = value.get("root", {}).get("physical_identity", {}).get("Linux", {})
    root_stat = os.stat(root, follow_symlinks=False)
    return (
        isinstance(value, dict)
        and set(value) == expected
        and valid_u32(value.get("schema_version")) and value["schema_version"] == 1
        and valid_id(value.get("source_store_id"), "ias_")
        and set(value.get("root", {})) == {"physical_path", "physical_identity"}
        and value["root"].get("physical_path") == root
        and set(value["root"].get("physical_identity", {})) == {"Linux"}
        and set(identity) == {"device_id", "inode"}
        and valid_u64(identity.get("device_id")) and identity["device_id"] == root_stat.st_dev
        and valid_u64(identity.get("inode")) and identity["inode"] == root_stat.st_ino
        and valid_timestamp(value.get("created_at"))
        and value.get("store_hash") == domain_hash("substrate.e3.installer-artifact-source-store.v1", "store", value, "store_hash")
    )

def validate_support_value(value):
    if not isinstance(value, dict) or set(value) != {"schema_version", "support_policy_version", "elf_execution_model", "elf_interpreter", "dynamic_loader_cache", "ordered_elf_dependencies", "ordered_present_common_files", "system_config_mount_target", "manifest_hash"}:
        return False
    model = value.get("elf_execution_model")
    if model != "StaticExec":
        if not isinstance(model, dict) or set(model) != {"StaticPie"}:
            return False
        pie = model["StaticPie"]
        if not isinstance(pie, dict) or set(pie) != {"dynamic_segment_file_offset", "dynamic_segment_byte_length", "rela_virtual_address", "rela_byte_length", "rela_entry_byte_length", "relative_relocation_count", "ordered_dynamic_entries_sha256"}:
            return False
        if not valid_u64(pie.get("dynamic_segment_file_offset")) or any(not valid_u64(pie.get(name)) or pie[name] <= 0 for name in ("dynamic_segment_byte_length", "rela_virtual_address", "rela_byte_length", "relative_relocation_count")) or not valid_u64(pie.get("rela_entry_byte_length")) or pie["rela_entry_byte_length"] != 24 or pie["rela_byte_length"] // 24 != pie["relative_relocation_count"] or not valid_digest(pie.get("ordered_dynamic_entries_sha256")):
            return False
    common_paths = ["/etc/hosts", "/etc/nsswitch.conf", "/etc/passwd", "/etc/group", "/etc/resolv.conf", "/etc/ssl/certs/ca-certificates.crt"]
    common = value.get("ordered_present_common_files")
    if not isinstance(common, list) or [item.get("absolute_path") if isinstance(item, dict) else None for item in common] != common_paths:
        return False
    for item in common:
        if set(item) != {"absolute_path", "device_id", "inode", "mode", "byte_length", "sha256"} or not valid_u64(item["device_id"]) or item["device_id"] <= 0 or not valid_u64(item["inode"]) or item["inode"] <= 0 or not valid_u32(item["mode"]) or item["mode"] & 0o022 or not valid_u64(item["byte_length"]) or not valid_digest(item["sha256"]):
            return False
    target = value.get("system_config_mount_target")
    if not isinstance(target, dict) or set(target) != {"absolute_path", "device_id", "inode", "mode", "owner_uid", "owner_gid", "ordered_entry_names"} or target.get("absolute_path") != "/etc/codex" or not valid_u64(target.get("device_id")) or target["device_id"] <= 0 or not valid_u64(target.get("inode")) or target["inode"] <= 0 or not valid_u32(target.get("mode")) or target["mode"] != 0o755 or not valid_u64(target.get("owner_uid")) or target["owner_uid"] != 0 or not valid_u64(target.get("owner_gid")) or target["owner_gid"] != 0 or target.get("ordered_entry_names") != []:
        return False
    return valid_u32(value.get("schema_version")) and value["schema_version"] == 1 and valid_u32(value.get("support_policy_version")) and value["support_policy_version"] == 1 and value.get("elf_interpreter") is None and value.get("dynamic_loader_cache") is None and value.get("ordered_elf_dependencies") == [] and value.get("manifest_hash") == domain_hash("substrate.e3.runtime-support-manifest.v1", "manifest", value, "manifest_hash")

def validate_entry_value(value):
    return (
        isinstance(value, dict)
        and set(value) == {"component", "installed_absolute_path", "device_id", "inode", "file_type", "mode", "owner_uid", "byte_length", "sha256", "runtime_support", "entry_hash"}
        and value.get("component") == "codex"
        and value.get("installed_absolute_path") == executable
        and valid_u64(value.get("device_id")) and value["device_id"] > 0
        and valid_u64(value.get("inode")) and value["inode"] > 0
        and value.get("file_type") == "regular"
        and valid_u32(value.get("mode")) and value["mode"] == 0o755
        and valid_u64(value.get("owner_uid")) and value["owner_uid"] == 0
        and valid_u64(value.get("byte_length")) and value["byte_length"] > 0
        and value.get("sha256") == executable_sha256 and valid_digest(value["sha256"])
        and validate_support_value(value.get("runtime_support"))
        and value.get("entry_hash") == domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", value, "entry_hash")
    )

def validate_record_value(value, source_store_id, expected_reference=None):
    build = value.get("build_input", {}).get("CodexOfficialArchive", {}) if isinstance(value, dict) else {}
    reference_matches = expected_reference is None or (
        value.get("source_store_id") == expected_reference.get("source_store_id")
        and value.get("source_record_id") == expected_reference.get("source_record_id")
        and value.get("revision") == expected_reference.get("revision")
        and value.get("record_hash") == expected_reference.get("record_hash")
    )
    predecessor = value.get("predecessor_ref") if isinstance(value, dict) else None
    predecessor_valid = predecessor is None if value.get("revision") == 1 else (
        isinstance(predecessor, dict)
        and set(predecessor) == {"source_store_id", "source_record_id", "revision", "record_hash"}
        and predecessor.get("source_store_id") == source_store_id
        and valid_id(predecessor.get("source_record_id"), "iar_")
        and valid_u64(predecessor.get("revision"))
        and predecessor["revision"] == value.get("revision") - 1
        and valid_digest(predecessor.get("record_hash"))
    )
    return (
        isinstance(value, dict)
        and set(value) == {"schema_version", "source_store_id", "source_record_id", "source_stream", "revision", "predecessor_ref", "build_input", "entries", "created_at", "record_hash"}
        and valid_u32(value.get("schema_version")) and value["schema_version"] == 1
        and value.get("source_store_id") == source_store_id
        and valid_id(value.get("source_record_id"), "iar_")
        and value.get("source_stream") == "Codex0125OfficialArchive"
        and valid_u64(value.get("revision")) and value["revision"] > 0
        and predecessor_valid
        and set(value.get("build_input", {})) == {"CodexOfficialArchive"}
        and set(build) == {"version", "target_triple", "archive_name", "archive_url", "archive_sha256", "archive_entry_path", "extracted_executable_sha256"}
        and build == {"version": "0.125.0", "target_triple": "x86_64-unknown-linux-musl", "archive_name": archive_name, "archive_url": archive_url, "archive_sha256": archive_sha256, "archive_entry_path": archive_entry, "extracted_executable_sha256": executable_sha256}
        and valid_digest(build.get("archive_sha256")) and valid_digest(build.get("extracted_executable_sha256"))
        and isinstance(value.get("entries"), list) and len(value["entries"]) == 1 and validate_entry_value(value["entries"][0])
        and valid_timestamp(value.get("created_at"))
        and value.get("record_hash") == domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", value, "record_hash")
        and reference_matches
    )

def validate_head_shape(value, source_store_id):
    reference = value.get("head_ref", {}) if isinstance(value, dict) else {}
    revision = value.get("head_revision") if isinstance(value, dict) else None
    return (
        isinstance(value, dict)
        and set(value) == {"schema_version", "source_store_id", "source_stream", "head_ref", "head_revision", "predecessor_head_hash", "updated_at", "head_hash"}
        and valid_u32(value.get("schema_version")) and value["schema_version"] == 1
        and value.get("source_store_id") == source_store_id
        and value.get("source_stream") == "Codex0125OfficialArchive"
        and isinstance(reference, dict) and set(reference) == {"source_store_id", "source_record_id", "revision", "record_hash"}
        and reference.get("source_store_id") == source_store_id
        and valid_id(reference.get("source_record_id"), "iar_")
        and valid_u64(revision) and revision > 0 and valid_u64(reference.get("revision")) and reference["revision"] == revision
        and valid_digest(reference.get("record_hash"))
        and ((revision == 1 and value.get("predecessor_head_hash") is None) or (revision > 1 and valid_digest(value.get("predecessor_head_hash"))))
        and valid_timestamp(value.get("updated_at"))
        and value.get("head_hash") == domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", value, "head_hash")
    )

def load_valid_store():
    path = os.path.join(root, "source-store.json")
    with open(path, "rb") as source:
        body = source.read()
    value = json.loads(body)
    if canonical(value) != body or not validate_store_value(value):
        raise RuntimeError("invalid Codex artifact source store")
    return value

def validate_head_record(value):
    if not validate_head_shape(value, load_valid_store()["source_store_id"]):
        return False
    reference = value["head_ref"]
    path = os.path.join(root, "records", stream_name, f"{reference['revision']:020d}-{reference['source_record_id']}.json")
    if not os.path.exists(path):
        return False
    with open(path, "rb") as source:
        body = source.read()
    record = json.loads(body)
    return canonical(record) == body and validate_record_value(record, value["source_store_id"], reference)

def fsync_dir(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)

def exact_write(path, value, immutable):
    body = canonical(value)
    if os.path.exists(path):
        with open(path, "rb") as existing:
            existing_body = existing.read()
        if existing_body == body:
            return body
        if immutable:
            raise RuntimeError("installer artifact source publication conflict")
    temp = os.path.join(os.path.dirname(path), f".e3-artifact-tmp.{uuid7()}.{'record' if immutable else 'head'}")
    descriptor = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
    try:
        os.write(descriptor, body)
        os.fchmod(descriptor, 0o640)
        if os.geteuid() == 0:
            os.fchown(descriptor, 0, substrate_gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    if immutable:
        try:
            os.link(temp, path, follow_symlinks=False)
        except FileExistsError:
            with open(path, "rb") as existing:
                if existing.read() != body:
                    raise
        os.unlink(temp)
    else:
        os.replace(temp, path)
    os.chmod(path, 0o640, follow_symlinks=False)
    fsync_dir(os.path.dirname(path))
    with open(path, "rb") as published:
        if published.read() != body:
            raise RuntimeError("installer artifact source readback mismatch")
    return body

def read_canonical_value(path, error):
    with open(path, "rb") as source:
        body = source.read()
    value = json.loads(body)
    if canonical(value) != body:
        raise RuntimeError(error)
    return body, value

def validate_recovery_transaction(contenders):
    if not any(item[3] in {"record", "head"} for item in contenders.values()):
        return
    store = load_valid_store()
    source_store_id = store["source_store_id"]
    head_path = os.path.join(root, "heads", stream_name + ".json")
    current_head = None
    current_head_body = None
    if os.path.exists(head_path):
        current_head_body, current_head = read_canonical_value(head_path, "invalid Codex artifact publication final")
        if not validate_head_record(current_head):
            raise RuntimeError("invalid Codex artifact publication final")

    pending_records = {}
    pending_head = None
    for final, (_, body, value, kind) in contenders.items():
        if kind == "record":
            if os.path.exists(final):
                current_body, current = read_canonical_value(final, "invalid Codex artifact publication final")
                if not validate_record_value(current, source_store_id) or current_body != body:
                    raise RuntimeError("Codex artifact publication temporary conflicts with final")
            else:
                pending_records[os.path.basename(final)] = (body, value)
        elif kind == "head":
            if current_head_body == body:
                continue
            pending_head = value

    if pending_head is None:
        validate_complete_record_chain(current_head)
        if pending_records:
            raise RuntimeError("incomplete Codex artifact recovery transaction")
        return

    if not validate_head_shape(pending_head, source_store_id):
        raise RuntimeError("invalid Codex artifact publication temporary")
    if current_head is None:
        if pending_head["predecessor_head_hash"] is not None or pending_head["head_revision"] != 1:
            raise RuntimeError("invalid initial Codex artifact head temporary")
        expected_predecessor_ref = None
    else:
        if pending_head["predecessor_head_hash"] != current_head["head_hash"] or pending_head["head_revision"] != current_head["head_revision"] + 1:
            raise RuntimeError("stale Codex artifact head temporary")
        expected_predecessor_ref = current_head["head_ref"]

    reference = pending_head["head_ref"]
    record_name = f"{reference['revision']:020d}-{reference['source_record_id']}.json"
    record_path = os.path.join(root, "records", stream_name, record_name)
    if record_name in pending_records:
        _, prospective_record = pending_records[record_name]
    elif os.path.exists(record_path):
        _, prospective_record = read_canonical_value(record_path, "invalid Codex artifact publication final")
    else:
        raise RuntimeError("Codex artifact head temporary lacks its immutable record")
    if not validate_record_value(prospective_record, source_store_id, reference):
        raise RuntimeError("Codex artifact head temporary names an invalid immutable record")
    if prospective_record.get("predecessor_ref") != expected_predecessor_ref:
        raise RuntimeError("Codex artifact record predecessor does not match the current head")
    validate_complete_record_chain(pending_head, pending_records)

def recover_publication():
    locations = [
        (root, "store"),
        (os.path.join(root, "records"), None),
        (os.path.join(root, "records", stream_name), "record"),
        (os.path.join(root, "heads"), "head"),
    ]
    pattern = re.compile(r"^\.e3-artifact-tmp\.([0-9a-f-]{36})\.(store|record|head)$")
    contenders = {}
    for directory, allowed_kind in locations:
        for name in os.listdir(directory):
            if not name.startswith(".e3-artifact-tmp."):
                continue
            match = pattern.fullmatch(name)
            if match is None or allowed_kind is None or match.group(2) != allowed_kind:
                raise RuntimeError("unrecognized Codex artifact publication temporary")
            parsed_uuid = uuid.UUID(match.group(1))
            if parsed_uuid.version != 7 or str(parsed_uuid) != match.group(1):
                raise RuntimeError("invalid Codex artifact publication temporary identity")
            temporary = os.path.join(directory, name)
            info = os.stat(temporary, follow_symlinks=False)
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or stat.S_IMODE(info.st_mode) != 0o640:
                raise RuntimeError("unsafe Codex artifact publication temporary")
            with open(temporary, "rb") as source:
                body = source.read()
            value = json.loads(body)
            if canonical(value) != body:
                raise RuntimeError("noncanonical Codex artifact publication temporary")
            kind = match.group(2)
            if kind == "store":
                final = os.path.join(root, "source-store.json")
                valid = validate_store_value(value)
            elif kind == "record":
                final = os.path.join(directory, f"{value.get('revision', 0):020d}-{value.get('source_record_id', '')}.json")
                valid = validate_record_value(value, load_valid_store()["source_store_id"])
            else:
                final = os.path.join(root, "heads", stream_name + ".json")
                valid = validate_head_shape(value, load_valid_store()["source_store_id"])
            if not valid:
                raise RuntimeError("invalid Codex artifact publication temporary")
            if final in contenders:
                raise RuntimeError("ambiguous Codex artifact publication temporaries")
            contenders[final] = (temporary, body, value, kind)
    validate_recovery_transaction(contenders)
    for final, (temporary, body, value, kind) in sorted(contenders.items(), key=lambda item: {"store": 0, "record": 1, "head": 2}[item[1][3]]):
        if os.path.exists(final):
            with open(final, "rb") as source:
                current_body = source.read()
            current = json.loads(current_body)
            current_valid = canonical(current) == current_body
            if kind == "store":
                current_valid = current_valid and validate_store_value(current)
            elif kind == "record":
                current_valid = current_valid and validate_record_value(current, load_valid_store()["source_store_id"])
            else:
                current_valid = current_valid and validate_head_record(current)
            if not current_valid:
                raise RuntimeError("invalid Codex artifact publication final")
            if current_body == body:
                os.unlink(temporary)
                fsync_dir(os.path.dirname(temporary))
                continue
            if kind != "head":
                raise RuntimeError("Codex artifact publication temporary conflicts with final")
        if kind == "head":
            reference = value["head_ref"]
            record_path = os.path.join(root, "records", stream_name, f"{reference['revision']:020d}-{reference['source_record_id']}.json")
            if not os.path.exists(record_path):
                raise RuntimeError("Codex artifact head temporary lacks its immutable record")
            if not validate_head_record(value):
                raise RuntimeError("Codex artifact head temporary names an invalid immutable record")
            predecessor = value["predecessor_head_hash"]
            if os.path.exists(final):
                with open(final, "rb") as source:
                    current = json.loads(source.read())
                if predecessor != current.get("head_hash") or value["head_revision"] != current.get("head_revision", 0) + 1:
                    raise RuntimeError("stale Codex artifact head temporary")
            elif predecessor is not None or value["head_revision"] != 1:
                raise RuntimeError("invalid initial Codex artifact head temporary")
            os.replace(temporary, final)
        else:
            os.link(temporary, final, follow_symlinks=False)
            os.unlink(temporary)
        fsync_dir(os.path.dirname(final))
        with open(final, "rb") as source:
            published_body = source.read()
            if published_body != body:
                raise RuntimeError("recovered Codex artifact publication readback mismatch")
        published = json.loads(published_body)
        if (kind == "store" and not validate_store_value(published)) or (kind == "record" and not validate_record_value(published, load_valid_store()["source_store_id"])) or (kind == "head" and not validate_head_record(published)):
            raise RuntimeError("recovered Codex artifact publication is incomplete")

def stage_publication(path, value, kind):
    body = canonical(value)
    temporary = os.path.join(os.path.dirname(path), f".e3-artifact-tmp.{uuid7()}.{kind}")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
    try:
        os.write(descriptor, body)
        os.fchmod(descriptor, 0o640)
        if os.geteuid() == 0:
            os.fchown(descriptor, 0, substrate_gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    return temporary, body

def commit_staged_publication(temporary, path, body, immutable):
    if immutable:
        try:
            os.link(temporary, path, follow_symlinks=False)
        except FileExistsError:
            with open(path, "rb") as source:
                if source.read() != body:
                    raise RuntimeError("Codex artifact immutable publication conflict")
        os.unlink(temporary)
    else:
        os.replace(temporary, path)
    fsync_dir(os.path.dirname(path))
    with open(path, "rb") as published:
        if published.read() != body:
            raise RuntimeError("Codex staged artifact publication readback mismatch")

def validate_complete_record_chain(head, pending_records=None):
    directory = os.path.join(root, "records", stream_name)
    records = {name for name in os.listdir(directory) if not name.startswith(".")}
    pending_records = {} if pending_records is None else pending_records
    if records.intersection(pending_records):
        raise RuntimeError("conflicting Codex artifact source record")
    records.update(pending_records)
    if any(re.fullmatch(r"[0-9]{20}-iar_[0-9a-f-]{36}\.json", name) is None for name in records):
        raise RuntimeError("unknown Codex artifact source record")
    if head is None:
        if records:
            raise RuntimeError("orphan Codex artifact source record")
        return
    store = load_valid_store()
    if not validate_head_shape(head, store["source_store_id"]):
        raise RuntimeError("invalid Codex artifact source head")
    reachable = set()
    reference = head["head_ref"]
    while reference is not None:
        name = f"{reference['revision']:020d}-{reference['source_record_id']}.json"
        if name in reachable or name not in records:
            raise RuntimeError("broken Codex artifact source predecessor chain")
        reachable.add(name)
        if name in pending_records:
            body, value = pending_records[name]
        else:
            with open(os.path.join(directory, name), "rb") as source:
                body = source.read()
            value = json.loads(body)
        if canonical(value) != body or not validate_record_value(value, store["source_store_id"], reference):
            raise RuntimeError("invalid Codex artifact source predecessor record")
        reference = value.get("predecessor_ref")
    if reachable != records:
        raise RuntimeError("orphan Codex artifact source record")

def file_support(path):
    st = os.stat(path, follow_symlinks=False)
    if not stat.S_ISREG(st.st_mode) or stat.S_IMODE(st.st_mode) & 0o022:
        raise RuntimeError(f"runtime support object is not regular: {path}")
    with open(path, "rb") as source:
        digest = hashlib.sha256(source.read()).hexdigest()
    return {
        "absolute_path": path,
        "device_id": st.st_dev,
        "inode": st.st_ino,
        "mode": stat.S_IMODE(st.st_mode),
        "byte_length": st.st_size,
        "sha256": digest,
    }

def elf_model(path):
    with open(path, "rb") as source:
        data = source.read()
    if len(data) < 64 or data[:16] != b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00" or struct.unpack_from("<H", data, 18)[0] != 62 or struct.unpack_from("<I", data, 20)[0] != 1 or struct.unpack_from("<H", data, 52)[0] != 64:
        raise RuntimeError("unsupported Codex ELF")
    elf_type = struct.unpack_from("<H", data, 16)[0]
    entry = struct.unpack_from("<Q", data, 24)[0]
    phoff = struct.unpack_from("<Q", data, 32)[0]
    phentsize, phnum = struct.unpack_from("<HH", data, 54)
    if phentsize != 56 or phnum == 0 or phoff + phentsize * phnum > len(data):
        raise RuntimeError("malformed Codex ELF program table")
    dynamic = None
    loads = []
    for index in range(phnum):
        offset = phoff + index * phentsize
        segment_type, flags = struct.unpack_from("<II", data, offset)
        file_offset, virtual_address = struct.unpack_from("<QQ", data, offset + 8)
        file_size, memory_size, alignment = struct.unpack_from("<QQQ", data, offset + 32)
        if file_size > memory_size or file_offset + file_size > len(data) or virtual_address + memory_size >= 1 << 64 or (alignment > 1 and ((alignment & (alignment - 1)) or file_offset % alignment != virtual_address % alignment)):
            raise RuntimeError("malformed Codex ELF segment")
        if segment_type == 1:
            loads.append((file_offset, virtual_address, file_size, memory_size, flags))
        if segment_type == 3:
            raise RuntimeError("Codex ELF has an interpreter")
        if segment_type == 2:
            if dynamic is not None:
                raise RuntimeError("Codex ELF has multiple dynamic segments")
            if file_size != memory_size:
                raise RuntimeError("malformed Codex ELF dynamic segment")
            dynamic = file_offset, virtual_address, file_size
    if not loads or not any(flags & 1 and entry >= virtual_address and entry < virtual_address + file_size for _, virtual_address, file_size, _, flags in loads):
        raise RuntimeError("Codex ELF has no usable executable entrypoint")
    if elf_type == 2 and dynamic is None:
        return "StaticExec"
    if elf_type != 3 or dynamic is None:
        raise RuntimeError("Codex ELF is not static")
    offset, dynamic_address, length = dynamic
    if length == 0 or length % 16 or offset + length > len(data):
        raise RuntimeError("malformed Codex ELF dynamic segment")
    allowed = {0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 21, 25, 26, 27, 28, 30, 0x6FFFFFFB, 0x6FFFFFF9, 0x6FFFFEF5}
    entries = []
    values = {}
    terminal = False
    for position in range(offset, offset + length, 16):
        tag, value = struct.unpack_from("<QQ", data, position)
        entries.append(data[position:position + 16])
        if tag not in allowed or terminal:
            raise RuntimeError("unsupported Codex ELF dynamic tag")
        if tag == 0:
            terminal = True
            break
        if tag in values:
            raise RuntimeError("duplicate Codex ELF dynamic tag")
        values[tag] = value
    terminal_end = offset + len(entries) * 16
    if not terminal or any(data[terminal_end:offset + length]) or values.get(9) != 24 or not values.get(8) or values[8] % 24 or values.get(0x6FFFFFF9) != values[8] // 24 or values.get(10) != 1 or values.get(11) != 24 or 5 not in values or 6 not in values or not ({4, 0x6FFFFEF5} & values.keys()) or values.get(30) != 8 or values.get(0x6FFFFFFB) != 0x08000001 or values.get(21) != 0 or ((25 in values) != (27 in values)) or ((26 in values) != (28 in values)):
        raise RuntimeError("unsupported Codex ELF relocation closure")
    def extent(address, byte_length):
        if address + byte_length >= 1 << 64:
            raise RuntimeError("Codex dynamic extent overflows")
        for file_offset, virtual_address, file_length, _, _ in loads:
            if address >= virtual_address and address + byte_length <= virtual_address + file_length:
                start = file_offset + address - virtual_address
                if start + byte_length <= len(data):
                    return data[start:start + byte_length]
        raise RuntimeError("Codex dynamic address escapes load segments")
    if extent(dynamic_address, length) != data[offset:offset + length] or extent(values[5], 1) != b"\x00" or any(extent(values[6], 24)):
        raise RuntimeError("Codex dynamic metadata extent mismatch")
    symbol_counts = []
    if 4 in values:
        buckets, chains = struct.unpack("<II", extent(values[4], 8))
        table = extent(values[4], 8 + 4 * (buckets + chains))
        if chains != 1 or any(struct.unpack_from("<I", table, position)[0] for position in range(8, len(table), 4)):
            raise RuntimeError("Codex ELF SysV hash exposes extra symbols")
        symbol_counts.append(chains)
    if 0x6FFFFEF5 in values:
        buckets, symbol_offset, bloom_words, _ = struct.unpack("<IIII", extent(values[0x6FFFFEF5], 16))
        prefix = extent(values[0x6FFFFEF5], 16 + 8 * bloom_words + 4 * buckets)
        bloom_end = 16 + 8 * bloom_words
        if not buckets or not bloom_words or bloom_words & (bloom_words - 1) or symbol_offset != 1 or any(prefix[16:bloom_end]) or any(struct.unpack_from("<I", prefix, position)[0] for position in range(bloom_end, len(prefix), 4)):
            raise RuntimeError("Codex ELF GNU hash exposes extra symbols")
        symbol_counts.append(symbol_offset)
    if not symbol_counts or any(count != 1 for count in symbol_counts):
        raise RuntimeError("Codex ELF dynamic symbol count is not exact")
    for tag in (3, 12, 13):
        if tag in values:
            extent(values[tag], 1)
    for address_tag, size_tag in ((25, 27), (26, 28)):
        if address_tag in values:
            if not values[size_tag]:
                raise RuntimeError("empty Codex dynamic array")
            extent(values[address_tag], values[size_tag])
    relocations = extent(values[7], values[8])
    if any(struct.unpack_from("<Q", relocations, position + 8)[0] != 8 for position in range(0, len(relocations), 24)):
        raise RuntimeError("Codex has a non-relative relocation")
    return {"StaticPie": {
        "dynamic_segment_file_offset": offset,
        "dynamic_segment_byte_length": length,
        "rela_virtual_address": values[7],
        "rela_byte_length": values[8],
        "rela_entry_byte_length": values[9],
        "relative_relocation_count": values[0x6FFFFFF9],
        "ordered_dynamic_entries_sha256": hashlib.sha256(b"".join(entries)).hexdigest(),
    }}

for directory in [root, os.path.join(root, "records"), os.path.join(root, "records", stream_name), os.path.join(root, "heads")]:
    os.chmod(directory, 0o750)
try:
    substrate_gid = grp.getgrnam("substrate").gr_gid
except KeyError:
    raise RuntimeError("missing substrate group for Codex artifact source")
if os.geteuid() == 0:
    for base, directories, files in os.walk(root):
        os.chown(base, 0, substrate_gid)
        for name in files:
            os.chown(os.path.join(base, name), 0, substrate_gid, follow_symlinks=False)

root_stat = os.stat(root, follow_symlinks=False)
if not stat.S_ISDIR(root_stat.st_mode) or os.path.realpath(root) != root:
    raise RuntimeError("installer artifact source root is not canonical")
recover_publication()
store_path = os.path.join(root, "source-store.json")
if os.path.exists(store_path):
    with open(store_path, "rb") as source:
        store_bytes = source.read()
    store = json.loads(store_bytes)
    if canonical(store) != store_bytes or not validate_store_value(store):
        raise RuntimeError("installer artifact source store is not canonical")
else:
    store = {
        "schema_version": 1,
        "source_store_id": "ias_" + uuid7(),
        "root": {"physical_path": root, "physical_identity": {"Linux": {"device_id": root_stat.st_dev, "inode": root_stat.st_ino}}},
        "created_at": timestamp(),
        "store_hash": "",
    }
    store["store_hash"] = domain_hash("substrate.e3.installer-artifact-source-store.v1", "store", store, "store_hash")
    temp_store = os.path.join(root, f".e3-artifact-tmp.{uuid7()}.store")
    store_body = canonical(store)
    descriptor = os.open(temp_store, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
    try:
        os.write(descriptor, store_body)
        os.fchmod(descriptor, 0o640)
        if os.geteuid() == 0:
            os.fchown(descriptor, 0, substrate_gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    try:
        os.link(temp_store, store_path, follow_symlinks=False)
    finally:
        os.unlink(temp_store)
    fsync_dir(root)

target_stat = os.stat(executable, follow_symlinks=False)
if not stat.S_ISREG(target_stat.st_mode) or stat.S_IMODE(target_stat.st_mode) != 0o755 or target_stat.st_uid != 0 or os.listxattr(executable, follow_symlinks=False):
    raise RuntimeError("installed Codex descriptor is not root-owned mode 0755")
with open(executable, "rb") as source:
    installed_digest = hashlib.sha256(source.read()).hexdigest()
if installed_digest != executable_sha256:
    raise RuntimeError("installed Codex digest mismatch")
system_target = os.stat("/etc/codex", follow_symlinks=False)
if not stat.S_ISDIR(system_target.st_mode) or stat.S_IMODE(system_target.st_mode) != 0o755 or system_target.st_uid != 0 or system_target.st_gid != 0 or os.listdir("/etc/codex"):
    raise RuntimeError("E3 system configuration mount target is not exact")
common_paths = ["/etc/hosts", "/etc/nsswitch.conf", "/etc/passwd", "/etc/group", "/etc/resolv.conf", "/etc/ssl/certs/ca-certificates.crt"]
support = {
    "schema_version": 1,
    "support_policy_version": 1,
    "elf_execution_model": elf_model(executable),
    "elf_interpreter": None,
    "dynamic_loader_cache": None,
    "ordered_elf_dependencies": [],
    "ordered_present_common_files": [file_support(path) for path in common_paths],
    "system_config_mount_target": {
        "absolute_path": "/etc/codex",
        "device_id": system_target.st_dev,
        "inode": system_target.st_ino,
        "mode": stat.S_IMODE(system_target.st_mode),
        "owner_uid": system_target.st_uid,
        "owner_gid": system_target.st_gid,
        "ordered_entry_names": [],
    },
    "manifest_hash": "",
}
support["manifest_hash"] = domain_hash("substrate.e3.runtime-support-manifest.v1", "manifest", support, "manifest_hash")
entry = {
    "component": "codex",
    "installed_absolute_path": executable,
    "device_id": target_stat.st_dev,
    "inode": target_stat.st_ino,
    "file_type": "regular",
    "mode": stat.S_IMODE(target_stat.st_mode),
    "owner_uid": target_stat.st_uid,
    "byte_length": target_stat.st_size,
    "sha256": installed_digest,
    "runtime_support": support,
    "entry_hash": "",
}
entry["entry_hash"] = domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", entry, "entry_hash")
build_input = {"CodexOfficialArchive": {
    "version": "0.125.0",
    "target_triple": "x86_64-unknown-linux-musl",
    "archive_name": archive_name,
    "archive_url": archive_url,
    "archive_sha256": archive_sha256,
    "archive_entry_path": archive_entry,
    "extracted_executable_sha256": executable_sha256,
}}
head_path = os.path.join(root, "heads", stream_name + ".json")
prior_head = None
prior_head_bytes = None
prior_record = None
if os.path.exists(head_path):
    with open(head_path, "rb") as source:
        prior_head_bytes = source.read()
    prior_head = json.loads(prior_head_bytes)
    if canonical(prior_head) != prior_head_bytes:
        raise RuntimeError("Codex installer source head is not canonical")
    reference = prior_head["head_ref"]
    prior_record_path = os.path.join(root, "records", stream_name, f"{reference['revision']:020d}-{reference['source_record_id']}.json")
    with open(prior_record_path, "rb") as source:
        prior_record_bytes = source.read()
    prior_record = json.loads(prior_record_bytes)
    if canonical(prior_record) != prior_record_bytes:
        raise RuntimeError("Codex installer source record is not canonical")
    validate_complete_record_chain(prior_head)
    if prior_record["build_input"] == build_input and prior_record["entries"] == [entry]:
        sys.exit(0)
else:
    validate_complete_record_chain(None)

revision = 1 if prior_head is None else prior_head["head_revision"] + 1
record = {
    "schema_version": 1,
    "source_store_id": store["source_store_id"],
    "source_record_id": "iar_" + uuid7(),
    "source_stream": "Codex0125OfficialArchive",
    "revision": revision,
    "predecessor_ref": None if prior_head is None else prior_head["head_ref"],
    "build_input": build_input,
    "entries": [entry],
    "created_at": timestamp(),
    "record_hash": "",
}
record["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", record, "record_hash")
record_path = os.path.join(root, "records", stream_name, f"{revision:020d}-{record['source_record_id']}.json")
reference = {
    "source_store_id": store["source_store_id"],
    "source_record_id": record["source_record_id"],
    "revision": revision,
    "record_hash": record["record_hash"],
}
head = {
    "schema_version": 1,
    "source_store_id": store["source_store_id"],
    "source_stream": "Codex0125OfficialArchive",
    "head_ref": reference,
    "head_revision": revision,
    "predecessor_head_hash": None if prior_head is None else prior_head["head_hash"],
    "updated_at": timestamp(),
    "head_hash": "",
}
head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", head, "head_hash")
record_temporary, record_body = stage_publication(record_path, record, "record")
head_temporary, head_body = stage_publication(head_path, head, "head")
if prior_head_bytes is not None:
    with open(head_path, "rb") as current:
        if current.read() != prior_head_bytes:
            raise RuntimeError("Codex installer source head CAS conflict")
commit_staged_publication(record_temporary, record_path, record_body, True)
commit_staged_publication(head_temporary, head_path, head_body, False)
fsync_dir(os.path.join(root, "records", stream_name))
fsync_dir(os.path.join(root, "records"))
fsync_dir(os.path.join(root, "heads"))
fsync_dir(root)
PY"#
}

fn builtin_packages_v1() -> Vec<PackageDefV1> {
    vec![
        PackageDefV1 {
            version: 1,
            name: "bun".to_string(),
            description: Some("Bun runtime (script install into world-deps prefix).".to_string()),
            runnable: true,
            entrypoints: vec!["bun".to_string()],
            platforms: Vec::new(),
            wrappers: Vec::new(),
            install: InstallDefV1 {
                method: InstallMethodV1::Script,
                apt: Vec::new(),
                pacman: Vec::new(),
                script: Some(
                    r#"#!/usr/bin/env bash
set -euo pipefail

world_deps_root="/var/lib/substrate/world-deps"
world_deps_bin="${world_deps_root}/bin"
bun_root="${world_deps_root}/bun"

mkdir -p "${world_deps_bin}"
mkdir -p "${bun_root}"

export BUN_INSTALL="${bun_root}"

if [ -x "${bun_root}/bin/bun" ]; then
  "${bun_root}/bin/bun" upgrade
else
  curl -fsSL https://bun.sh/install | bash
fi

ln -sf "${bun_root}/bin/bun" "${world_deps_bin}/bun"
"#
                    .to_string(),
                ),
                script_path: None,
                manual_instructions: None,
            },
            probe: Some(ProbeDefV1 {
                command: "bun --version".to_string(),
            }),
            definition_path: None,
        },
        PackageDefV1 {
            version: 1,
            name: "node".to_string(),
            description: Some("Node.js runtime via apt.".to_string()),
            runnable: true,
            entrypoints: vec!["node".to_string()],
            platforms: Vec::new(),
            wrappers: Vec::new(),
            install: InstallDefV1 {
                method: InstallMethodV1::Apt,
                apt: vec![AptSpecV1 {
                    name: "nodejs".to_string(),
                    version: None,
                }],
                pacman: Vec::new(),
                script: None,
                script_path: None,
                manual_instructions: None,
            },
            probe: Some(ProbeDefV1 {
                command: "node --version".to_string(),
            }),
            definition_path: None,
        },
        PackageDefV1 {
            version: 1,
            name: "npm".to_string(),
            description: Some("npm CLI via apt.".to_string()),
            runnable: true,
            entrypoints: vec!["npm".to_string(), "npx".to_string()],
            platforms: Vec::new(),
            wrappers: Vec::new(),
            install: InstallDefV1 {
                method: InstallMethodV1::Apt,
                apt: vec![AptSpecV1 {
                    name: "npm".to_string(),
                    version: None,
                }],
                pacman: Vec::new(),
                script: None,
                script_path: None,
                manual_instructions: None,
            },
            probe: Some(ProbeDefV1 {
                command: "npm --version && npx --version".to_string(),
            }),
            definition_path: None,
        },
        PackageDefV1 {
            version: 1,
            name: "codex-runtime".to_string(),
            description: Some(
                "Official Codex Linux runtime from GitHub Releases; verified self-contained in the guest, so no wider Node/npm bundle is required.".to_string(),
            ),
            runnable: true,
            entrypoints: vec!["codex".to_string()],
            platforms: Vec::new(),
            wrappers: Vec::new(),
            install: InstallDefV1 {
                method: InstallMethodV1::Script,
                apt: Vec::new(),
                pacman: Vec::new(),
                script: Some(codex_runtime_install_script_template_v1().to_string()),
                script_path: None,
                manual_instructions: None,
            },
            probe: Some(ProbeDefV1 {
                command: "codex --version".to_string(),
            }),
            definition_path: None,
        },
    ]
}

fn builtin_bundles_v1() -> Vec<BundleDefV1> {
    vec![BundleDefV1 {
        version: 1,
        name: "node-runtime".to_string(),
        description: Some("Node.js + npm bundle.".to_string()),
        platforms: Vec::new(),
        packages: vec!["node".to_string(), "npm".to_string()],
    }]
}

pub(crate) fn load_inventory_dir_v1(dir: &Path, platform: HostPlatform) -> Result<InventoryViewV1> {
    let packages_dir = dir.join("packages");
    let bundles_dir = dir.join("bundles");

    let mut view = InventoryViewV1::default();
    if packages_dir.is_dir() {
        let pkgs = load_package_dir_v1(&packages_dir, platform)
            .with_context(|| format!("failed to read {}", packages_dir.display()))?;
        view.packages.extend(pkgs);
    }
    if bundles_dir.is_dir() {
        let bundles = load_bundle_dir_v1(&bundles_dir, platform)
            .with_context(|| format!("failed to read {}", bundles_dir.display()))?;
        view.bundles.extend(bundles);
    }
    view.validate_no_collisions()?;
    Ok(view)
}

fn load_package_dir_v1(
    dir: &Path,
    platform: HostPlatform,
) -> Result<BTreeMap<String, PackageDefV1>> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry.context("read_dir entry")?;
        let path = entry.path();
        if !is_yaml_file(&path) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| {
                config_model::user_error(format!("invalid package filename: {}", path.display()))
            })?
            .to_string();

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut parsed: PackageDefV1 = serde_yaml::from_str(&raw).map_err(|err| {
            config_model::user_error(format!(
                "invalid YAML in {}: {}",
                path.display(),
                err.to_string().trim()
            ))
        })?;
        parsed.definition_path = Some(path.clone());
        validate_package_v1(&path, &stem, &parsed, platform)?;
        if is_visible_on_platform(&parsed.platforms, platform)? {
            out.insert(parsed.name.clone(), parsed);
        }
    }
    Ok(out)
}

fn load_bundle_dir_v1(dir: &Path, platform: HostPlatform) -> Result<BTreeMap<String, BundleDefV1>> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry.context("read_dir entry")?;
        let path = entry.path();
        if !is_yaml_file(&path) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| {
                config_model::user_error(format!("invalid bundle filename: {}", path.display()))
            })?
            .to_string();
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let parsed: BundleDefV1 = serde_yaml::from_str(&raw).map_err(|err| {
            config_model::user_error(format!(
                "invalid YAML in {}: {}",
                path.display(),
                err.to_string().trim()
            ))
        })?;
        validate_bundle_v1(&path, &stem, &parsed, platform)?;
        if is_visible_on_platform(&parsed.platforms, platform)? {
            out.insert(parsed.name.clone(), parsed);
        }
    }
    Ok(out)
}

fn is_yaml_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("yaml") | Some("yml")
    )
}

fn validate_package_v1(
    path: &Path,
    expected_name: &str,
    pkg: &PackageDefV1,
    platform: HostPlatform,
) -> Result<()> {
    if pkg.version != 1 {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: version must be 1 (got {})",
            path.display(),
            pkg.version
        )));
    }
    if pkg.name.trim().is_empty() {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: name must be a non-empty string",
            path.display()
        )));
    }
    if pkg.name != expected_name {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: name '{}' must match filename '{}.yaml'",
            path.display(),
            pkg.name,
            expected_name
        )));
    }
    if pkg.runnable && pkg.entrypoints.is_empty() {
        return Err(config_model::user_error(format!(
            "invalid package schema in {}: runnable=true requires a non-empty entrypoints list",
            path.display()
        )));
    }
    for entrypoint in &pkg.entrypoints {
        if entrypoint.trim().is_empty() {
            return Err(config_model::user_error(format!(
                "invalid package schema in {}: entrypoints must be non-empty strings",
                path.display()
            )));
        }
    }
    match pkg.install.method {
        InstallMethodV1::Apt => {
            if pkg.install.apt.is_empty() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=apt requires a non-empty install.apt list",
                    path.display()
                )));
            }
            for spec in &pkg.install.apt {
                if spec.name.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: install.apt[].name must be non-empty",
                        path.display()
                    )));
                }
            }
        }
        InstallMethodV1::Pacman => {
            if pkg.install.pacman.is_empty() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman requires a non-empty install.pacman list",
                    path.display()
                )));
            }
            if !pkg.install.apt.is_empty() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman must not define install.apt",
                    path.display()
                )));
            }
            if pkg.install.script.is_some() || pkg.install.script_path.is_some() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman must not define install.script or install.script_path",
                    path.display()
                )));
            }
            if pkg
                .install
                .manual_instructions
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
            {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman must not define install.manual_instructions",
                    path.display()
                )));
            }
            if pkg.runnable {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman packages must not be runnable",
                    path.display()
                )));
            }
            if !pkg.entrypoints.is_empty() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman packages must not define entrypoints",
                    path.display()
                )));
            }
            if !pkg.wrappers.is_empty() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman packages must not define wrappers",
                    path.display()
                )));
            }
            if pkg.probe.is_some() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=pacman packages must not define probe",
                    path.display()
                )));
            }
            for package_name in &pkg.install.pacman {
                if package_name.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: install.pacman[] must contain non-empty strings",
                        path.display()
                    )));
                }
            }
        }
        InstallMethodV1::Script => {
            if pkg.install.script.is_none() && pkg.install.script_path.is_none() {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=script requires install.script or install.script_path",
                    path.display()
                )));
            }
        }
        InstallMethodV1::Manual => {
            if pkg
                .install
                .manual_instructions
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                return Err(config_model::user_error(format!(
                    "invalid package schema in {}: install.method=manual requires non-empty install.manual_instructions",
                    path.display()
                )));
            }
        }
    }
    for wrapper in &pkg.wrappers {
        if wrapper.name.trim().is_empty() {
            return Err(config_model::user_error(format!(
                "invalid package schema in {}: wrappers[].name must be non-empty",
                path.display()
            )));
        }
        if !pkg.entrypoints.iter().any(|e| e == &wrapper.name) {
            return Err(config_model::user_error(format!(
                "invalid package schema in {}: wrappers[].name '{}' must be listed in entrypoints[]",
                path.display(),
                wrapper.name
            )));
        }
        match &wrapper.kind {
            WrapperKindV1::BashFunction(def) => {
                if def.bash_source.trim().is_empty() || def.function.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: wrappers[].kind=bash_function requires non-empty bash_source and function",
                        path.display()
                    )));
                }
            }
            WrapperKindV1::BashSourceExec(def) => {
                if def.bash_source.trim().is_empty() || def.exec.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: wrappers[].kind=bash_source_exec requires non-empty bash_source and exec",
                        path.display()
                    )));
                }
            }
            WrapperKindV1::ShEnvExec(def) => {
                if def.exec.trim().is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: wrappers[].kind=sh_env_exec requires non-empty exec",
                        path.display()
                    )));
                }
                if def.env.is_empty() {
                    return Err(config_model::user_error(format!(
                        "invalid package schema in {}: wrappers[].kind=sh_env_exec requires non-empty env",
                        path.display()
                    )));
                }
                for key in def.env.keys() {
                    if key.trim().is_empty() {
                        return Err(config_model::user_error(format!(
                            "invalid package schema in {}: wrappers[].kind=sh_env_exec requires non-empty env keys",
                            path.display()
                        )));
                    }
                }
            }
        }
    }

    let _ = is_visible_on_platform(&pkg.platforms, platform)?;
    Ok(())
}

fn validate_bundle_v1(
    path: &Path,
    expected_name: &str,
    bundle: &BundleDefV1,
    platform: HostPlatform,
) -> Result<()> {
    if bundle.version != 1 {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: version must be 1 (got {})",
            path.display(),
            bundle.version
        )));
    }
    if bundle.name.trim().is_empty() {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: name must be a non-empty string",
            path.display()
        )));
    }
    if bundle.name != expected_name {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: name '{}' must match filename '{}.yaml'",
            path.display(),
            bundle.name,
            expected_name
        )));
    }
    if bundle.packages.is_empty() || bundle.packages.iter().any(|p| p.trim().is_empty()) {
        return Err(config_model::user_error(format!(
            "invalid bundle schema in {}: packages must be a non-empty list of non-empty strings",
            path.display()
        )));
    }
    let _ = is_visible_on_platform(&bundle.platforms, platform)?;
    Ok(())
}

fn is_visible_on_platform(platforms: &[String], platform: HostPlatform) -> Result<bool> {
    if platforms.is_empty() {
        return Ok(true);
    }
    let mut parsed = Vec::with_capacity(platforms.len());
    for raw in platforms {
        let Some(p) = parse_platform(raw) else {
            return Err(config_model::user_error(format!(
                "invalid platforms entry '{raw}'; expected one of: linux, macos, windows"
            )));
        };
        parsed.push(p);
    }
    Ok(platform.matches(&parsed))
}

pub(crate) fn merge_inventory_layer_v1(into: &mut InventoryViewV1, layer: InventoryViewV1) {
    for (name, pkg) in layer.packages {
        into.packages.insert(name, pkg);
    }
    for (name, bundle) in layer.bundles {
        into.bundles.insert(name, bundle);
    }
}

pub(crate) fn summarize_inventory_v1(view: &InventoryViewV1) -> Vec<InventoryListItemSummaryV1> {
    let mut out = Vec::new();
    for pkg in view.packages.values() {
        out.push(InventoryListItemSummaryV1 {
            kind: "package".to_string(),
            name: pkg.name.clone(),
            enabled: None,
            world: None,
            remediation: None,
            runnable: Some(pkg.runnable),
            method: Some(pkg.install.method.clone()),
            entrypoints: pkg.entrypoints.clone(),
            platforms: pkg.platforms.clone(),
            description: pkg.description.clone(),
        });
    }
    for bundle in view.bundles.values() {
        out.push(InventoryListItemSummaryV1 {
            kind: "bundle".to_string(),
            name: bundle.name.clone(),
            enabled: None,
            world: None,
            remediation: None,
            runnable: None,
            method: None,
            entrypoints: Vec::new(),
            platforms: bundle.platforms.clone(),
            description: bundle.description.clone(),
        });
    }
    out.sort_by(|a, b| (a.kind.as_str(), a.name.as_str()).cmp(&(b.kind.as_str(), b.name.as_str())));
    out
}

pub(crate) fn find_workspace_inventory_chain(cwd: &Path, stop_at: Option<&Path>) -> Vec<PathBuf> {
    let mut layers = Vec::new();
    for dir in cwd.ancestors() {
        let candidate = dir.join(".substrate").join("deps");
        if candidate.is_dir() {
            layers.push(candidate);
        }
        if stop_at.is_some_and(|stop| stop == dir) {
            break;
        }
    }
    layers.reverse();
    layers
}

#[cfg(test)]
mod e3c_tests {
    use super::render_codex_installer_artifact_source_v1;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;
    use std::process::Command;

    fn minimal_static_exec() -> Vec<u8> {
        let mut bytes = vec![0_u8; 120];
        bytes[..16].copy_from_slice(b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0");
        bytes[16..18].copy_from_slice(&2_u16.to_le_bytes());
        bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
        bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        bytes[24..32].copy_from_slice(&0x400000_u64.to_le_bytes());
        bytes[32..40].copy_from_slice(&64_u64.to_le_bytes());
        bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
        bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
        bytes[56..58].copy_from_slice(&1_u16.to_le_bytes());
        bytes[64..68].copy_from_slice(&1_u32.to_le_bytes());
        bytes[68..72].copy_from_slice(&5_u32.to_le_bytes());
        bytes[80..88].copy_from_slice(&0x400000_u64.to_le_bytes());
        bytes[96..104].copy_from_slice(&120_u64.to_le_bytes());
        bytes[104..112].copy_from_slice(&120_u64.to_le_bytes());
        bytes[112..120].copy_from_slice(&0x1000_u64.to_le_bytes());
        bytes
    }

    #[test]
    fn e3c_codex_artifact_publication_rejects_incomplete_staged_record_and_head() {
        let rendered = render_codex_installer_artifact_source_v1();
        let python_start = rendered.find("<<'PY'\n").expect("Python heredoc") + 7;
        let helpers_end = rendered[python_start..]
            .find("def file_support(path):")
            .map(|offset| python_start + offset)
            .expect("publisher helper boundary");
        let mut program = rendered[python_start..helpers_end].to_string();
        program.push_str(
            r#"
os.makedirs(os.path.join(root, "records", stream_name), mode=0o750)
os.makedirs(os.path.join(root, "heads"), mode=0o750)
substrate_gid = os.getgid()
assert valid_u64(0x100000000) and not valid_u32(0x100000000) and not valid_u64(False)
root_info = os.stat(root, follow_symlinks=False)
store = {"schema_version": 1, "source_store_id": "ias_018f0892-cc48-7a56-b711-9f17a2a81586", "root": {"physical_path": root, "physical_identity": {"Linux": {"device_id": root_info.st_dev, "inode": root_info.st_ino}}}, "created_at": "2026-09-11T12:00:00.000000Z", "store_hash": ""}
store["store_hash"] = domain_hash("substrate.e3.installer-artifact-source-store.v1", "store", store, "store_hash")
store_temp = os.path.join(root, f".e3-artifact-tmp.{uuid7()}.store")
with open(store_temp, "wb") as destination:
    destination.write(canonical(store))
os.chmod(store_temp, 0o640)
recover_publication()
assert os.path.exists(os.path.join(root, "source-store.json"))
record = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_record_id": "iar_018f0892-cc48-7a56-b711-9f17a2a81587", "source_stream": "Codex0125OfficialArchive", "revision": 1, "predecessor_ref": None, "build_input": {}, "entries": [], "created_at": "2026-09-11T12:00:00.000000Z", "record_hash": ""}
record["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", record, "record_hash")
reference = {"source_store_id": store["source_store_id"], "source_record_id": record["source_record_id"], "revision": 1, "record_hash": record["record_hash"]}
head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "Codex0125OfficialArchive", "head_ref": reference, "head_revision": 1, "predecessor_head_hash": None, "updated_at": "2026-09-11T12:00:00.000000Z", "head_hash": ""}
head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", head, "head_hash")
record_path = os.path.join(root, "records", stream_name, f"{record['revision']:020d}-{record['source_record_id']}.json")
head_path = os.path.join(root, "heads", stream_name + ".json")
stage_publication(record_path, record, "record")
stage_publication(head_path, head, "head")
recover_publication()
assert os.path.exists(record_path) and os.path.exists(head_path)
"#,
        );
        let root = tempfile::tempdir().expect("publisher root");
        let output = Command::new("python3")
            .arg("-c")
            .arg(program)
            .arg(root.path())
            .args([
                "/tmp/codex",
                "archive",
                "url",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "entry",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ])
            .output()
            .expect("run publisher recovery simulation");
        assert!(
            !output.status.success(),
            "incomplete publisher recovery unexpectedly succeeded"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("invalid Codex artifact publication temporary"),
            "unexpected publisher recovery failure: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e3c_codex_artifact_recovery_promotes_only_complete_bound_objects() {
        let rendered = render_codex_installer_artifact_source_v1();
        let python_start = rendered.find("<<'PY'\n").expect("Python heredoc") + 7;
        let helpers_end = rendered[python_start..]
            .find("def file_support(path):")
            .map(|offset| python_start + offset)
            .expect("publisher helper boundary");
        let mut program = rendered[python_start..helpers_end].to_string();
        program.push_str(
            r#"
os.makedirs(os.path.join(root, "records", stream_name), mode=0o750)
os.makedirs(os.path.join(root, "heads"), mode=0o750)
substrate_gid = os.getgid()
assert valid_u64(0x100000000) and not valid_u32(0x100000000) and not valid_u64(False)
root_info = os.stat(root, follow_symlinks=False)
store = {"schema_version": 1, "source_store_id": "ias_018f0892-cc48-7a56-b711-9f17a2a81586", "root": {"physical_path": root, "physical_identity": {"Linux": {"device_id": root_info.st_dev, "inode": root_info.st_ino}}}, "created_at": "2026-09-11T12:00:00.000000Z", "store_hash": ""}
store["store_hash"] = domain_hash("substrate.e3.installer-artifact-source-store.v1", "store", store, "store_hash")
store_temp = os.path.join(root, f".e3-artifact-tmp.{uuid7()}.store")
with open(store_temp, "wb") as destination:
    destination.write(canonical(store))
os.chmod(store_temp, 0o640)
recover_publication()
common_paths = ["/etc/hosts", "/etc/nsswitch.conf", "/etc/passwd", "/etc/group", "/etc/resolv.conf", "/etc/ssl/certs/ca-certificates.crt"]
common = [{"absolute_path": path, "device_id": 1, "inode": index + 1, "mode": 0o644, "byte_length": 1, "sha256": "c" * 64} for index, path in enumerate(common_paths)]
support = {"schema_version": 1, "support_policy_version": 1, "elf_execution_model": "StaticExec", "elf_interpreter": None, "dynamic_loader_cache": None, "ordered_elf_dependencies": [], "ordered_present_common_files": common, "system_config_mount_target": {"absolute_path": "/etc/codex", "device_id": 1, "inode": 20, "mode": 0o755, "owner_uid": 0, "owner_gid": 0, "ordered_entry_names": []}, "manifest_hash": ""}
support["manifest_hash"] = domain_hash("substrate.e3.runtime-support-manifest.v1", "manifest", support, "manifest_hash")
entry = {"component": "codex", "installed_absolute_path": executable, "device_id": 1, "inode": 30, "file_type": "regular", "mode": 0o755, "owner_uid": 0, "byte_length": 1, "sha256": executable_sha256, "runtime_support": support, "entry_hash": ""}
entry["entry_hash"] = domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", entry, "entry_hash")
build = {"CodexOfficialArchive": {"version": "0.125.0", "target_triple": "x86_64-unknown-linux-musl", "archive_name": archive_name, "archive_url": archive_url, "archive_sha256": archive_sha256, "archive_entry_path": archive_entry, "extracted_executable_sha256": executable_sha256}}
record = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_record_id": "iar_018f0892-cc48-7a56-b711-9f17a2a81587", "source_stream": "Codex0125OfficialArchive", "revision": 1, "predecessor_ref": None, "build_input": build, "entries": [entry], "created_at": "2026-09-11T12:00:00.000000Z", "record_hash": ""}
record["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", record, "record_hash")
reference = {"source_store_id": store["source_store_id"], "source_record_id": record["source_record_id"], "revision": 1, "record_hash": record["record_hash"]}
head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "Codex0125OfficialArchive", "head_ref": reference, "head_revision": 1, "predecessor_head_hash": None, "updated_at": "2026-09-11T12:00:00.000000Z", "head_hash": ""}
head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", head, "head_hash")
record_path = os.path.join(root, "records", stream_name, f"{record['revision']:020d}-{record['source_record_id']}.json")
head_path = os.path.join(root, "heads", stream_name + ".json")
stage_publication(record_path, record, "record")
stage_publication(head_path, head, "head")
recover_publication()
assert os.path.exists(record_path) and os.path.exists(head_path)

record_only = json.loads(json.dumps(record))
record_only["source_record_id"] = "iar_018f0892-cc48-7a56-b711-9f17a2a81588"
record_only["revision"] = 2
record_only["predecessor_ref"] = reference
record_only["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", record_only, "record_hash")
record_only_path = os.path.join(root, "records", stream_name, f"{record_only['revision']:020d}-{record_only['source_record_id']}.json")
record_only_temporary, _ = stage_publication(record_only_path, record_only, "record")
try:
    recover_publication()
except RuntimeError as error:
    assert "incomplete Codex artifact recovery transaction" in str(error)
else:
    raise AssertionError("record-only Codex recovery was promoted")
assert not os.path.exists(record_only_path)
os.unlink(record_only_temporary)

broken = json.loads(json.dumps(record_only))
broken["source_record_id"] = "iar_018f0892-cc48-7a56-b711-9f17a2a81589"
broken["predecessor_ref"] = {"source_store_id": store["source_store_id"], "source_record_id": "iar_018f0892-cc48-7a56-b711-9f17a2a81590", "revision": 1, "record_hash": "f" * 64}
broken["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", broken, "record_hash")
broken_reference = {"source_store_id": store["source_store_id"], "source_record_id": broken["source_record_id"], "revision": 2, "record_hash": broken["record_hash"]}
broken_head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "Codex0125OfficialArchive", "head_ref": broken_reference, "head_revision": 2, "predecessor_head_hash": head["head_hash"], "updated_at": "2026-09-11T12:00:01.000000Z", "head_hash": ""}
broken_head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", broken_head, "head_hash")
broken_path = os.path.join(root, "records", stream_name, f"{broken['revision']:020d}-{broken['source_record_id']}.json")
broken_record_temporary, _ = stage_publication(broken_path, broken, "record")
broken_head_temporary, _ = stage_publication(head_path, broken_head, "head")
try:
    recover_publication()
except RuntimeError as error:
    assert "record predecessor does not match the current head" in str(error)
else:
    raise AssertionError("broken Codex predecessor chain was promoted")
assert not os.path.exists(broken_path)
os.unlink(broken_record_temporary)
os.unlink(broken_head_temporary)

boolean_record = json.loads(json.dumps(record_only))
boolean_record["source_record_id"] = "iar_018f0892-cc48-7a56-b711-9f17a2a81591"
boolean_record["entries"][0]["owner_uid"] = False
boolean_record["entries"][0]["entry_hash"] = domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", boolean_record["entries"][0], "entry_hash")
boolean_record["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", boolean_record, "record_hash")
boolean_reference = {"source_store_id": store["source_store_id"], "source_record_id": boolean_record["source_record_id"], "revision": 2, "record_hash": boolean_record["record_hash"]}
boolean_head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "Codex0125OfficialArchive", "head_ref": boolean_reference, "head_revision": 2, "predecessor_head_hash": head["head_hash"], "updated_at": "2026-09-11T12:00:02.000000Z", "head_hash": ""}
boolean_head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", boolean_head, "head_hash")
boolean_path = os.path.join(root, "records", stream_name, f"{boolean_record['revision']:020d}-{boolean_record['source_record_id']}.json")
boolean_record_temporary, _ = stage_publication(boolean_path, boolean_record, "record")
boolean_head_temporary, _ = stage_publication(head_path, boolean_head, "head")
try:
    recover_publication()
except RuntimeError as error:
    assert "invalid Codex artifact publication temporary" in str(error)
else:
    raise AssertionError("boolean Codex integer field was accepted")
assert not os.path.exists(boolean_path)
os.unlink(boolean_record_temporary)
os.unlink(boolean_head_temporary)

bad = json.loads(json.dumps(record))
bad["source_store_id"] = "ias_018f0892-cc48-7a56-b711-9f17a2a81588"
bad["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", bad, "record_hash")
bad_path = os.path.join(root, "records", stream_name, f"{bad['revision']:020d}-{bad['source_record_id']}.json")
stage_publication(bad_path, bad, "record")
try:
    recover_publication()
except RuntimeError as error:
    assert "invalid Codex artifact publication temporary" in str(error)
else:
    raise AssertionError("mismatched temporary was promoted")
with open(record_path, "rb") as source:
    assert source.read() == canonical(record)
"#,
        );
        let root = tempfile::tempdir().expect("publisher root");
        let output = Command::new("python3")
            .arg("-c")
            .arg(program)
            .arg(root.path())
            .args([
                "/tmp/codex",
                "codex-x86_64-unknown-linux-musl.tar.gz",
                "https://example.invalid/codex.tar.gz",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "codex-x86_64-unknown-linux-musl",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ])
            .output()
            .expect("run strict publisher recovery simulation");
        assert!(
            output.status.success(),
            "strict publisher recovery simulation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e3c_substrate_publisher_is_unavailable_without_both_e3d_artifacts() {
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lifecycle = repository.join("scripts/linux/world-lifecycle.sh");
        let root = tempfile::tempdir().expect("fake installation root");
        let output = Command::new("bash")
            .arg("-c")
            .arg(
                r#"source "$1"
sudo_cmd() { "$@"; }
FAKE_ROOT="$2"
publish_substrate_artifact_source_v1
"#,
            )
            .arg("e3c-substrate-publisher-test")
            .arg(lifecycle)
            .arg(root.path())
            .output()
            .expect("exercise bounded Substrate publisher");
        assert!(
            !output.status.success(),
            "missing E3-D artifacts must make the publication helper unavailable"
        );
    }

    #[test]
    fn e3c_installed_home_bootstrap_recovery_rejects_incomplete_temporary() {
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lifecycle = repository.join("scripts/linux/world-lifecycle.sh");
        let root = tempfile::tempdir().expect("fake installation root");
        let accepted_home = root.path().join("accepted-home");
        std::fs::create_dir(&accepted_home).expect("accepted home");
        let account = std::env::var("USER").unwrap_or_else(|_| "tester".to_string());
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        let encode = |value: &str| URL_SAFE_NO_PAD.encode(value.as_bytes());
        let frame = format!(
            "domain=substrate.install_bootstrap_context\nversion=1\nselected_host_prefix={}\nhost_substrate_home={}\nhost_substrate_root={}\nprincipal_kind=unix\nprincipal_account={}\nprincipal_uid={}\n",
            encode(accepted_home.to_str().unwrap()),
            encode(accepted_home.to_str().unwrap()),
            encode(root.path().to_str().unwrap()),
            encode(&account),
            uid,
        );
        let commitment = format!("{:x}", Sha256::digest(frame.as_bytes()));
        let carrier = URL_SAFE_NO_PAD
            .encode(format!("{frame}host_context_commitment={commitment}\n").as_bytes());
        let output = Command::new("bash")
            .arg("-c")
            .arg(
                r#"source "$1"
sudo_cmd() { "$@"; }
FAKE_ROOT="$2"
INSTALL_BOOTSTRAP_CONTEXT_V1="$3"
INSTALL_BOOTSTRAP_COMMITMENT="$4"
INSTALL_BOOTSTRAP_ACCOUNT="$5"
INSTALL_BOOTSTRAP_UID="$6"
INSTALL_BOOTSTRAP_PRIMARY_GID="$7"
INSTALL_PREFIX="$8"
publish_installed_home_bootstrap_v1
authority_root="${FAKE_ROOT}/var/lib/substrate/install-bootstrap-authority-v1"
python3 - "${authority_root}" <<'PY'
import hashlib, json, os
root = __import__('sys').argv[1]
value = {"schema_version": 1, "record_hash": ""}
value["record_hash"] = hashlib.sha256(json.dumps({"domain": "substrate.e3.installed-accepted-home-bootstrap.v1", "record": {"schema_version": 1}}, separators=(",", ":"), sort_keys=True).encode()).hexdigest()
path = os.path.join(root, "records", ".e3-bootstrap-tmp.018f0892-cc48-7a56-b711-9f17a2a81589.record")
with open(path, "wb") as destination:
    destination.write(json.dumps(value, separators=(",", ":"), sort_keys=True).encode())
os.chmod(path, 0o640)
PY
if publish_installed_home_bootstrap_v1; then
    echo "incomplete bootstrap temporary was promoted" >&2
    exit 91
fi
test -f "${authority_root}/active.json"
"#,
            )
            .arg("e3c-bootstrap-recovery-test")
            .arg(lifecycle)
            .arg(root.path())
            .arg(carrier)
            .arg(commitment)
            .arg(account)
            .arg(uid.to_string())
            .arg(gid.to_string())
            .arg(&accepted_home)
            .output()
            .expect("exercise bootstrap publication recovery");
        assert!(
            output.status.success(),
            "bootstrap recovery check failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn e3c_substrate_artifact_recovery_promotes_complete_bound_objects() {
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lifecycle =
            std::fs::read_to_string(repository.join("scripts/linux/world-lifecycle.sh"))
                .expect("read lifecycle source");
        let publisher_start = lifecycle
            .find("publish_substrate_artifact_source_v1() {")
            .expect("Substrate publisher");
        let python_start = lifecycle[publisher_start..]
            .find("<<'PY'\n")
            .map(|offset| publisher_start + offset + 7)
            .expect("Substrate publisher Python");
        let helpers_end = lifecycle[python_start..]
            .find("def file_support(path, recorded_path=None):")
            .map(|offset| python_start + offset)
            .expect("Substrate recovery helper boundary");
        let mut program = lifecycle[python_start..helpers_end].to_string();
        program.push_str(
            r#"
os.makedirs(os.path.join(root, "records", "substrate-source-build"), mode=0o750)
os.makedirs(os.path.join(root, "heads"), mode=0o750)
substrate_gid = os.getgid()
assert valid_u64(0x100000000) and not valid_u32(0x100000000) and not valid_u64(False)
root_info = os.stat(root, follow_symlinks=False)
store = {"schema_version": 1, "source_store_id": "ias_018f0892-cc48-7a56-b711-9f17a2a81586", "root": {"physical_path": root, "physical_identity": {"Linux": {"device_id": root_info.st_dev, "inode": root_info.st_ino}}}, "created_at": "2026-09-11T12:00:00.000000Z", "store_hash": ""}
store["store_hash"] = domain_hash("substrate.e3.installer-artifact-source-store.v1", "store", store, "store_hash")
store_temp = os.path.join(root, f".e3-artifact-tmp.{uuid7()}.store")
with open(store_temp, "wb") as destination:
    destination.write(canonical(store))
os.chmod(store_temp, 0o640)
recover_publication()
common_paths = ["/etc/hosts", "/etc/nsswitch.conf", "/etc/passwd", "/etc/group", "/etc/resolv.conf", "/etc/ssl/certs/ca-certificates.crt"]
common = [{"absolute_path": path, "device_id": 1, "inode": index + 1, "mode": 0o644, "byte_length": 1, "sha256": "c" * 64} for index, path in enumerate(common_paths)]
support = {"schema_version": 1, "support_policy_version": 1, "elf_execution_model": "StaticExec", "elf_interpreter": None, "dynamic_loader_cache": None, "ordered_elf_dependencies": [], "ordered_present_common_files": common, "system_config_mount_target": {"absolute_path": "/etc/codex", "device_id": 1, "inode": 20, "mode": 0o755, "owner_uid": 0, "owner_gid": 0, "ordered_entry_names": []}, "manifest_hash": ""}
support["manifest_hash"] = domain_hash("substrate.e3.runtime-support-manifest.v1", "manifest", support, "manifest_hash")
entries = []
for index, (component, installed, digest) in enumerate((("substrate-gateway", "/usr/local/lib/substrate/e3/substrate-gateway", "d" * 64), ("substrate-world-entry", "/usr/local/lib/substrate/e3/substrate-world-entry", "e" * 64))):
    entry = {"component": component, "installed_absolute_path": installed, "device_id": 1, "inode": 30 + index, "file_type": "regular", "mode": 0o755, "owner_uid": 0, "byte_length": 1, "sha256": digest, "runtime_support": support, "entry_hash": ""}
    entry["entry_hash"] = domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", entry, "entry_hash")
    entries.append(entry)
build = {"SubstrateSourceBuild": {"source_commit": source_commit, "source_tree": source_tree, "cargo_lock_sha256": lock_hash, "rustc_version": rustc_version, "target_triple": target, "profile": profile}}
record = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_record_id": "iar_018f0892-cc48-7a56-b711-9f17a2a81587", "source_stream": "SubstrateSourceBuild", "revision": 1, "predecessor_ref": None, "build_input": build, "entries": entries, "created_at": "2026-09-11T12:00:00.000000Z", "record_hash": ""}
record["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", record, "record_hash")
reference = {"source_store_id": store["source_store_id"], "source_record_id": record["source_record_id"], "revision": 1, "record_hash": record["record_hash"]}
head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "SubstrateSourceBuild", "head_ref": reference, "head_revision": 1, "predecessor_head_hash": None, "updated_at": "2026-09-11T12:00:00.000000Z", "head_hash": ""}
head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", head, "head_hash")
record_path = os.path.join(root, "records", "substrate-source-build", f"{record['revision']:020d}-{record['source_record_id']}.json")
head_path = os.path.join(root, "heads", "substrate-source-build.json")
stage_publication(record_path, record, "record")
stage_publication(head_path, head, "head")
recover_publication()
assert os.path.exists(record_path) and os.path.exists(head_path)

record_only = json.loads(json.dumps(record))
record_only["source_record_id"] = "iar_018f0892-cc48-7a56-b711-9f17a2a81588"
record_only["revision"] = 2
record_only["predecessor_ref"] = reference
record_only["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", record_only, "record_hash")
record_only_path = os.path.join(root, "records", "substrate-source-build", f"{record_only['revision']:020d}-{record_only['source_record_id']}.json")
record_only_temporary, _ = stage_publication(record_only_path, record_only, "record")
try:
    recover_publication()
except RuntimeError as error:
    assert "incomplete Substrate artifact recovery transaction" in str(error)
else:
    raise AssertionError("record-only Substrate recovery was promoted")
assert not os.path.exists(record_only_path)
os.unlink(record_only_temporary)

broken = json.loads(json.dumps(record_only))
broken["source_record_id"] = "iar_018f0892-cc48-7a56-b711-9f17a2a81589"
broken["predecessor_ref"] = {"source_store_id": store["source_store_id"], "source_record_id": "iar_018f0892-cc48-7a56-b711-9f17a2a81590", "revision": 1, "record_hash": "f" * 64}
broken["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", broken, "record_hash")
broken_reference = {"source_store_id": store["source_store_id"], "source_record_id": broken["source_record_id"], "revision": 2, "record_hash": broken["record_hash"]}
broken_head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "SubstrateSourceBuild", "head_ref": broken_reference, "head_revision": 2, "predecessor_head_hash": head["head_hash"], "updated_at": "2026-09-11T12:00:01.000000Z", "head_hash": ""}
broken_head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", broken_head, "head_hash")
broken_path = os.path.join(root, "records", "substrate-source-build", f"{broken['revision']:020d}-{broken['source_record_id']}.json")
broken_record_temporary, _ = stage_publication(broken_path, broken, "record")
broken_head_temporary, _ = stage_publication(head_path, broken_head, "head")
try:
    recover_publication()
except RuntimeError as error:
    assert "record predecessor does not match the current head" in str(error)
else:
    raise AssertionError("broken Substrate predecessor chain was promoted")
assert not os.path.exists(broken_path)
os.unlink(broken_record_temporary)
os.unlink(broken_head_temporary)

boolean_record = json.loads(json.dumps(record_only))
boolean_record["source_record_id"] = "iar_018f0892-cc48-7a56-b711-9f17a2a81591"
boolean_record["entries"][0]["owner_uid"] = False
boolean_record["entries"][0]["entry_hash"] = domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", boolean_record["entries"][0], "entry_hash")
boolean_record["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", boolean_record, "record_hash")
boolean_reference = {"source_store_id": store["source_store_id"], "source_record_id": boolean_record["source_record_id"], "revision": 2, "record_hash": boolean_record["record_hash"]}
boolean_head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "SubstrateSourceBuild", "head_ref": boolean_reference, "head_revision": 2, "predecessor_head_hash": head["head_hash"], "updated_at": "2026-09-11T12:00:02.000000Z", "head_hash": ""}
boolean_head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", boolean_head, "head_hash")
boolean_path = os.path.join(root, "records", "substrate-source-build", f"{boolean_record['revision']:020d}-{boolean_record['source_record_id']}.json")
boolean_record_temporary, _ = stage_publication(boolean_path, boolean_record, "record")
boolean_head_temporary, _ = stage_publication(head_path, boolean_head, "head")
try:
    recover_publication()
except RuntimeError as error:
    assert "invalid Substrate artifact publication temporary" in str(error)
else:
    raise AssertionError("boolean Substrate integer field was accepted")
assert not os.path.exists(boolean_path)
os.unlink(boolean_record_temporary)
os.unlink(boolean_head_temporary)
"#,
        );
        let root = tempfile::tempdir().expect("Substrate publisher root");
        let output = Command::new("python3")
            .arg("-c")
            .arg(program)
            .arg(root.path())
            .args([
                "/usr/local/lib/substrate/e3/substrate-gateway",
                "/usr/local/lib/substrate/e3/substrate-world-entry",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "rustc 1.89.0",
                "x86_64-unknown-linux-musl",
                "release",
                "1",
            ])
            .output()
            .expect("run strict Substrate recovery simulation");
        assert!(
            output.status.success(),
            "strict Substrate recovery simulation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn e3c_substrate_publisher_accepts_bounded_complete_static_artifacts() {
        use std::os::unix::fs::PermissionsExt;

        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lifecycle = repository.join("scripts/linux/world-lifecycle.sh");
        let root = tempfile::tempdir().expect("fake installation root");
        for path in ["usr/local/lib/substrate/e3", "etc/codex", "etc/ssl/certs"] {
            std::fs::create_dir_all(root.path().join(path)).expect("fixture directory");
        }
        for path in [
            "etc/hosts",
            "etc/nsswitch.conf",
            "etc/passwd",
            "etc/group",
            "etc/resolv.conf",
            "etc/ssl/certs/ca-certificates.crt",
        ] {
            std::fs::write(root.path().join(path), b"fixture\n").expect("support fixture");
        }
        for name in ["substrate-gateway", "substrate-world-entry"] {
            let path = root.path().join("usr/local/lib/substrate/e3").join(name);
            std::fs::write(&path, minimal_static_exec()).expect("static artifact fixture");
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .expect("executable fixture mode");
        }
        let output = Command::new("fakeroot")
            .arg("--")
            .arg("bash")
            .arg("-c")
            .arg(
                r#"source "$1"
sudo_cmd() { "$@"; }
FAKE_ROOT="$2"
SUBSTRATE_SOURCE_COMMIT=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
SUBSTRATE_SOURCE_TREE=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
SUBSTRATE_CARGO_LOCK_SHA256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
SUBSTRATE_RUSTC_VERSION='rustc 1.89.0'
SUBSTRATE_SOURCE_TARGET=x86_64-unknown-linux-musl
SUBSTRATE_SOURCE_PROFILE=release
SUBSTRATE_SOURCE_BUILD_PERFORMED=1
chown -R 0:0 "${FAKE_ROOT}"
publish_substrate_artifact_source_v1
publish_substrate_artifact_source_v1
test -f "${FAKE_ROOT}/var/lib/substrate/runtime-artifacts-v1/heads/substrate-source-build.json"
"#,
            )
            .arg("e3c-bounded-substrate-publisher-test")
            .arg(lifecycle)
            .arg(root.path())
            .output()
            .expect("exercise bounded Substrate publisher");
        assert!(
            output.status.success(),
            "bounded Substrate publisher failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn e3c_both_publishers_reject_malformed_elf_and_extra_dynamic_symbols() {
        use std::os::unix::fs::PermissionsExt;

        let Ok(pinned) = std::env::var("SUBSTRATE_E3C_PINNED_CODEX") else {
            return;
        };
        let mut bytes = std::fs::read(pinned).expect("pinned Codex executable");
        assert_eq!(&bytes[0x23c..0x240], &1_u32.to_le_bytes());
        bytes[0x23c..0x240].copy_from_slice(&2_u32.to_le_bytes());
        let artifact = tempfile::NamedTempFile::new().expect("mutated artifact");
        std::fs::write(artifact.path(), bytes).expect("mutated artifact bytes");
        std::fs::set_permissions(artifact.path(), std::fs::Permissions::from_mode(0o755))
            .expect("mutated artifact mode");
        let malformed = tempfile::NamedTempFile::new().expect("malformed artifact");
        let mut malformed_bytes = minimal_static_exec();
        malformed_bytes[64..68].copy_from_slice(&0_u32.to_le_bytes());
        std::fs::write(malformed.path(), malformed_bytes).expect("malformed artifact bytes");
        std::fs::set_permissions(malformed.path(), std::fs::Permissions::from_mode(0o755))
            .expect("malformed artifact mode");

        let rendered = render_codex_installer_artifact_source_v1();
        let python_start = rendered.find("<<'PY'\n").expect("Python heredoc") + 7;
        let helpers_end = rendered[python_start..]
            .find("for directory in [root,")
            .map(|offset| python_start + offset)
            .expect("Codex ELF helper boundary");
        let mut codex_program = rendered[python_start..helpers_end].to_string();
        codex_program.push_str(
            r#"
for candidate, message in ((executable, "extra symbols"), (os.environ["MALFORMED_ELF"], "entrypoint")):
    try:
        elf_model(candidate)
    except RuntimeError as error:
        assert message in str(error)
    else:
        raise AssertionError("Codex publisher accepted malformed ELF authority")
"#,
        );
        let root = tempfile::tempdir().expect("Codex helper root");
        let codex = Command::new("python3")
            .arg("-c")
            .arg(codex_program)
            .env("MALFORMED_ELF", malformed.path())
            .arg(root.path())
            .arg(artifact.path())
            .args(["archive", "url", "a", "entry", "b"])
            .output()
            .expect("run Codex ELF helper");
        assert!(
            codex.status.success(),
            "Codex ELF helper check failed: {}",
            String::from_utf8_lossy(&codex.stderr)
        );

        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lifecycle =
            std::fs::read_to_string(repository.join("scripts/linux/world-lifecycle.sh"))
                .expect("read lifecycle source");
        let publisher_start = lifecycle
            .find("publish_substrate_artifact_source_v1() {")
            .expect("Substrate publisher");
        let substrate_start = lifecycle[publisher_start..]
            .find("<<'PY'\n")
            .map(|offset| publisher_start + offset + 7)
            .expect("Substrate publisher Python");
        let substrate_end = lifecycle[substrate_start..]
            .find("os.makedirs(os.path.join(root, \"records\"")
            .map(|offset| substrate_start + offset)
            .expect("Substrate ELF helper boundary");
        let mut substrate_program = lifecycle[substrate_start..substrate_end].to_string();
        substrate_program.push_str(
            r#"
for candidate, message in ((gateway, "extra symbols"), (os.environ["MALFORMED_ELF"], "entrypoint")):
    try:
        file_record(candidate)
    except RuntimeError as error:
        assert message in str(error)
    else:
        raise AssertionError("Substrate publisher accepted malformed ELF authority")
"#,
        );
        let substrate_root = tempfile::tempdir().expect("Substrate helper root");
        let substrate = Command::new("fakeroot")
            .arg("--")
            .arg("bash")
            .arg("-c")
            .arg("artifact=$1; shift; chown 0:0 \"${artifact}\" \"${MALFORMED_ELF}\"; exec python3 \"$@\"")
            .env("MALFORMED_ELF", malformed.path())
            .arg("e3c-substrate-elf-test")
            .arg(artifact.path())
            .arg("-c")
            .arg(substrate_program)
            .arg(substrate_root.path())
            .arg(artifact.path())
            .args([
                "/usr/local/lib/substrate/e3/substrate-world-entry",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "rustc 1.89.0",
                "x86_64-unknown-linux-musl",
                "release",
                "1",
            ])
            .output()
            .expect("run Substrate ELF helper");
        assert!(
            substrate.status.success(),
            "Substrate ELF helper check failed: {}",
            String::from_utf8_lossy(&substrate.stderr)
        );
    }

    #[test]
    fn e3c_production_lifecycle_does_not_claim_e3d_artifact_publication() {
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lifecycle =
            std::fs::read_to_string(repository.join("scripts/linux/world-lifecycle.sh"))
                .expect("read lifecycle source");
        let provision =
            std::fs::read_to_string(repository.join("scripts/linux/world-provision.sh"))
                .expect("read provision source");
        let install_start = lifecycle
            .find("install_linux_managed_state() {")
            .expect("install function");
        let install_end = lifecycle[install_start..]
            .find("\nrestore_linux_managed_state() {")
            .map(|offset| install_start + offset)
            .expect("install function boundary");

        assert!(
            !lifecycle[install_start..install_end].contains("publish_substrate_artifact_source_v1")
        );
        assert!(!provision.contains("SUBSTRATE_SOURCE_BUILD_PERFORMED"));
        assert!(!provision.contains("SUBSTRATE_SOURCE_TARGET"));
    }
}
