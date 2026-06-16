#!/usr/bin/env bash
set -u

socket="${1:-/run/substrate.sock}"
group="${2:-substrate}"
tag="substrate-apply-socket-acl"

log_warn() {
  if command -v systemd-cat >/dev/null 2>&1; then
    printf '%s\n' "$*" | systemd-cat -t "${tag}" -p warning || true
  else
    printf '%s: %s\n' "${tag}" "$*" >&2
  fi
}

safe_exit() {
  exit 0
}

if [[ ! -S "${socket}" ]]; then
  log_warn "socket path is not an AF_UNIX socket: ${socket}"
  safe_exit
fi

if ! command -v setfacl >/dev/null 2>&1; then
  log_warn "setfacl not found; immediate first-run socket access bridge is degraded"
  safe_exit
fi

group_entry="$(getent group "${group}" 2>/dev/null || true)"
if [[ -z "${group_entry}" ]]; then
  log_warn "group not found: ${group}; cannot project named-user socket ACLs"
  safe_exit
fi

IFS=: read -r _ _ group_gid explicit_members <<< "${group_entry}"
if [[ -z "${group_gid:-}" ]]; then
  log_warn "group entry for ${group} did not include a GID"
  safe_exit
fi

declare -A users=()

IFS=',' read -ra member_arr <<< "${explicit_members:-}"
for user in "${member_arr[@]}"; do
  [[ -z "${user}" ]] && continue
  if getent passwd "${user}" >/dev/null 2>&1; then
    users["${user}"]=1
  else
    log_warn "skipping unresolved explicit group member: ${user}"
  fi
done

passwd_entries="$(getent passwd 2>/dev/null || true)"
if [[ -z "${passwd_entries}" ]]; then
  log_warn "passwd enumeration unavailable; primary-GID ${group} users may not receive ACL bridge"
else
  while IFS=: read -r user _ _ gid _; do
    [[ -z "${user}" ]] && continue
    if [[ "${gid}" == "${group_gid}" ]]; then
      users["${user}"]=1
    fi
  done <<< "${passwd_entries}"
fi

acl_file="$(mktemp 2>/dev/null || true)"
if [[ -z "${acl_file}" ]]; then
  log_warn "mktemp failed; cannot build socket ACL file for ${socket}"
  safe_exit
fi
trap 'rm -f "${acl_file}"' EXIT

{
  echo "user::rw-"
  echo "group::rw-"
  echo "mask::rw-"
  echo "other::---"
  if ((${#users[@]} > 0)); then
    while IFS= read -r user; do
      [[ -z "${user}" ]] && continue
      echo "user:${user}:rw-"
    done < <(printf '%s\n' "${!users[@]}" | LC_ALL=C sort)
  fi
} > "${acl_file}"

if ! setfacl --set-file="${acl_file}" "${socket}" >/dev/null 2>&1; then
  log_warn "failed to apply socket ACL to ${socket}; immediate first-run access bridge is degraded"
  safe_exit
fi

safe_exit
