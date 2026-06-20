#!/usr/bin/env bash
set -u

mode="socket"
case "${1:-}" in
  --socket)
    mode="socket"
    shift
    ;;
  --directory-traverse)
    mode="directory-traverse"
    shift
    ;;
  --tree-readonly)
    mode="tree-readonly"
    shift
    ;;
esac

target="${1:-/run/substrate.sock}"
group="${2:-substrate}"
tag="substrate-apply-acl-bridge"

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

declare -A users=()

require_setfacl() {
  local degraded_message="$1"
  if ! command -v setfacl >/dev/null 2>&1; then
    log_warn "${degraded_message}"
    safe_exit
  fi
}

collect_authorized_users() {
  local purpose="$1"
  local group_entry=""
  local group_gid=""
  local explicit_members=""
  local passwd_entries=""

  group_entry="$(getent group "${group}" 2>/dev/null || true)"
  if [[ -z "${group_entry}" ]]; then
    log_warn "group not found: ${group}; cannot project named-user ACLs for ${purpose}"
    safe_exit
  fi

  IFS=: read -r _ _ group_gid explicit_members <<< "${group_entry}"
  if [[ -z "${group_gid:-}" ]]; then
    log_warn "group entry for ${group} did not include a GID"
    safe_exit
  fi

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
    return
  fi

  while IFS=: read -r user _ _ gid _; do
    [[ -z "${user}" ]] && continue
    if [[ "${gid}" == "${group_gid}" ]]; then
      users["${user}"]=1
    fi
  done <<< "${passwd_entries}"
}

sorted_user_list() {
  printf '%s\n' "${!users[@]}" | LC_ALL=C sort
}

build_acl_spec() {
  local permission="$1"
  local prefix="${2:-u}"
  local first=1
  local spec=""
  while IFS= read -r user; do
    [[ -z "${user}" ]] && continue
    if [[ ${first} -eq 0 ]]; then
      spec+=","
    fi
    spec+="${prefix}:${user}:${permission}"
    first=0
  done < <(sorted_user_list)
  printf '%s\n' "${spec}"
}

apply_socket_acl() {
  local socket="$1"
  if [[ ! -S "${socket}" ]]; then
    log_warn "socket path is not an AF_UNIX socket: ${socket}"
    safe_exit
  fi

  require_setfacl "setfacl not found; immediate first-run socket access bridge is degraded"
  collect_authorized_users "socket ${socket}"

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
      done < <(sorted_user_list)
    fi
  } > "${acl_file}"

  if ! setfacl --set-file="${acl_file}" "${socket}" >/dev/null 2>&1; then
    log_warn "failed to apply socket ACL to ${socket}; immediate first-run access bridge is degraded"
    safe_exit
  fi
}

apply_directory_traverse_acl() {
  local directory="$1"
  if [[ ! -d "${directory}" ]]; then
    log_warn "directory-traverse ACL target is not a directory: ${directory}"
    safe_exit
  fi

  require_setfacl "setfacl not found; immediate world-deps parent traversal bridge is degraded"
  collect_authorized_users "directory ${directory}"

  local acl_spec=""
  acl_spec="$(build_acl_spec "--x")"
  if [[ -z "${acl_spec}" ]]; then
    safe_exit
  fi

  if ! setfacl -m "${acl_spec}" "${directory}" >/dev/null 2>&1; then
    log_warn "failed to apply directory-traverse ACL to ${directory}; immediate world-deps parent traversal bridge is degraded"
    safe_exit
  fi
}

apply_tree_readonly_acl() {
  local tree_root="$1"
  if [[ ! -d "${tree_root}" ]]; then
    log_warn "tree-readonly ACL target is not a directory: ${tree_root}"
    safe_exit
  fi

  require_setfacl "setfacl not found; immediate world-deps ACL bridge is degraded"
  collect_authorized_users "tree ${tree_root}"

  local dir_acl_spec=""
  local dir_default_acl_spec=""
  local file_acl_spec=""
  local exec_file_acl_spec=""

  dir_acl_spec="$(build_acl_spec "r-x")"
  dir_default_acl_spec="$(build_acl_spec "r-x" "d:u")"
  file_acl_spec="$(build_acl_spec "r--")"
  exec_file_acl_spec="$(build_acl_spec "r-x")"

  if [[ -z "${dir_acl_spec}" ]]; then
    safe_exit
  fi

  if ! find "${tree_root}" -xdev -type d -exec setfacl -m "${dir_acl_spec}" {} + >/dev/null 2>&1; then
    log_warn "failed to apply readonly directory ACLs to ${tree_root}; immediate world-deps ACL bridge is degraded"
    safe_exit
  fi
  if ! find "${tree_root}" -xdev -type d -exec setfacl -m "${dir_default_acl_spec}" {} + >/dev/null 2>&1; then
    log_warn "failed to apply default directory ACLs to ${tree_root}; future world-deps children may require a shell reload"
    safe_exit
  fi
  if [[ -n "${file_acl_spec}" ]] && ! find "${tree_root}" -xdev -type f -exec setfacl -m "${file_acl_spec}" {} + >/dev/null 2>&1; then
    log_warn "failed to apply readonly file ACLs to ${tree_root}; immediate world-deps ACL bridge is degraded"
    safe_exit
  fi
  if [[ -n "${exec_file_acl_spec}" ]] && ! find "${tree_root}" -xdev -type f -perm /111 -exec setfacl -m "${exec_file_acl_spec}" {} + >/dev/null 2>&1; then
    log_warn "failed to apply executable file ACLs to ${tree_root}; immediate world-deps ACL bridge is degraded"
    safe_exit
  fi
}

case "${mode}" in
  socket)
    apply_socket_acl "${target}"
    ;;
  directory-traverse)
    apply_directory_traverse_acl "${target}"
    ;;
  tree-readonly)
    apply_tree_readonly_acl "${target}"
    ;;
  *)
    log_warn "unsupported ACL bridge mode: ${mode}"
    safe_exit
    ;;
esac

safe_exit
