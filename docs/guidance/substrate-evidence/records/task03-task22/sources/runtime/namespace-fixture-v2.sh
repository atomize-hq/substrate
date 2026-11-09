#!/usr/bin/env bash
# PREVIEW ONLY. Materialize and invoke only under the owner's exact effect/build release.
set -Eeuo pipefail
TASK22_PRODUCT=__TASK22_PRIVATE_HOME__/__Active_code/.worktrees/substrate-task22-r08-production-reader-20261101
TASK22_PRIVATE_CASE=${1:?one released case is required}
case "$TASK22_PRIVATE_CASE" in
  attach|resume|stop|custody-pending|custody-corrupt|custody-substituted|custody-revision|publisher-failure|preflight|preflight-resume|preflight-owner|preflight-legacy|preflight-custody|legacy-cli|service-package|writer-unit|unit|world|transport-client|docs) ;;
  *) exit 90 ;;
esac
test "$(id -u)" = 0
test "$$" = 1  # The required unshare --pid --fork invocation, before any fixture writes.
TASK22_PRIVATE_ROOT="$TASK22_PRODUCT/runtime/joined/$TASK22_PRIVATE_CASE"
test ! -e "$TASK22_PRIVATE_ROOT"
case "${TASK22_SHELL_TEST_BINARY:?}" in "$TASK22_PRODUCT"/target/debug/deps/substrate_shell-*) ;; *) exit 91;; esac
case "${TASK22_SERVICE_TEST_BINARY:?}" in "$TASK22_PRODUCT"/target/debug/deps/world_service-*) ;; *) exit 91;; esac
test -x "$TASK22_SHELL_TEST_BINARY"
test -x "$TASK22_SERVICE_TEST_BINARY"
TASK22_SUBSTRATE_BINARY="$TASK22_PRODUCT/target/debug/substrate"
test -x "$TASK22_SUBSTRATE_BINARY"
case "$TASK22_PRIVATE_CASE" in
  world)
    case "${TASK22_WORLD_TEST_BINARY:?}" in "$TASK22_PRODUCT"/target/debug/deps/world-*) ;; *) exit 91;; esac
    test -x "$TASK22_WORLD_TEST_BINARY"
    ;;
  transport-client)
    case "${TASK22_TRANSPORT_CLIENT_TEST_BINARY:?}" in "$TASK22_PRODUCT"/target/debug/deps/transport_api_client-*) ;; *) exit 91;; esac
    test -x "$TASK22_TRANSPORT_CLIENT_TEST_BINARY"
    ;;
  docs|legacy-cli|service-package)
    test -d __TASK22_PRIVATE_HOME__/.cargo/registry
    test -x __TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/cargo
    test -x __TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/rustc
    test -x __TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/rustdoc
    ;;
esac
TASK22_WORKSPACE_TMPFS=0
TASK22_CARGO_REGISTRY_MOUNT=0
TASK22_RUN_BIND_ATTEMPTED=0
TASK22_RUN_BIND_COMPLETE=0
TASK22_WORKSPACE_ALIAS_MOUNT=0
umask 077
test -d "$TASK22_PRODUCT/runtime/joined"
mkdir -- "$TASK22_PRIVATE_ROOT"
cleanup() {
  TASK22_EXIT_RESULT=$?
  TASK22_REMOVE_CASE=1
  trap - EXIT INT TERM
  kill -TERM -1 2>/dev/null || true
  sleep 0.1
  kill -KILL -1 2>/dev/null || true
  wait 2>/dev/null || true
  if test "$TASK22_CARGO_REGISTRY_MOUNT" = 1; then
    if ! umount "$TASK22_PRIVATE_ROOT/cargo-home/registry"; then
      TASK22_EXIT_RESULT=92
      TASK22_REMOVE_CASE=0
    fi
  fi
  if test "$TASK22_WORKSPACE_ALIAS_MOUNT" = 1; then
    if ! umount /run/task22-owner/workspace; then
      TASK22_EXIT_RESULT=92
      TASK22_REMOVE_CASE=0
    fi
  fi
  if test "$TASK22_RUN_BIND_ATTEMPTED" = 1 && test "$TASK22_RUN_BIND_COMPLETE" != 1; then
    # Recursive bind may partially install a child alias. Do not guess/unmount
    # an unknown /run path or recursively remove CASE; namespace exit releases it.
    TASK22_EXIT_RESULT=92
    TASK22_REMOVE_CASE=0
  fi
  if test "$TASK22_WORKSPACE_TMPFS" = 1; then
    if ! umount "$TASK22_PRIVATE_ROOT/run/task22-owner/workspace"; then
      TASK22_EXIT_RESULT=92
      TASK22_REMOVE_CASE=0
    fi
  fi
  # Never recurse through a remaining same-filesystem RW registry alias.
  if test "$TASK22_REMOVE_CASE" = 1; then
    if ! rm -rf --one-file-system -- "$TASK22_PRIVATE_ROOT"; then TASK22_EXIT_RESULT=92; fi
  else
    printf '%s\n' "cleanup retained exact case after mount-release failure: $TASK22_PRIVATE_ROOT" >&2
  fi
  exit "$TASK22_EXIT_RESULT"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mount --make-rprivate /
umask 077
mkdir -p "$TASK22_PRIVATE_ROOT"/{etc,varlib,vartmp,run,tmp,cgroup,control,cargo-home/registry}
mkdir -p "$TASK22_PRIVATE_ROOT/run"/{task22-root/home/.config,task22-root/home/.cache,task22-root/home/.local/share,task22-root/tmp,task22-owner/home/.config,task22-owner/home/.cache,task22-owner/home/.local/share,task22-owner/workspace,task22-owner/unit-runtime,task22-owner/tmp,task22-service-runtime/substrate/worlds}
chmod 0755 "$TASK22_PRIVATE_ROOT" "$TASK22_PRIVATE_ROOT/etc" "$TASK22_PRIVATE_ROOT/run"
chown -R 1000:1000 "$TASK22_PRIVATE_ROOT/control" "$TASK22_PRIVATE_ROOT/run/task22-owner" "$TASK22_PRIVATE_ROOT/cargo-home"
chmod 0700 "$TASK22_PRIVATE_ROOT/control" "$TASK22_PRIVATE_ROOT/run/task22-owner"
chmod 1777 "$TASK22_PRIVATE_ROOT/tmp" "$TASK22_PRIVATE_ROOT/vartmp"
cat >"$TASK22_PRIVATE_ROOT/etc/passwd" <<EOF
root:x:0:0:private fixture root:/run/task22-root/home:/bin/bash
task22-owner:x:1000:1000:private native fixture:/run/task22-owner/home:/bin/bash
EOF
cat >"$TASK22_PRIVATE_ROOT/etc/group" <<'EOF'
root:x:0:
substrate:x:1000:task22-owner
EOF
chmod 0644 "$TASK22_PRIVATE_ROOT/etc/passwd" "$TASK22_PRIVATE_ROOT/etc/group"
mount --bind "$TASK22_PRIVATE_ROOT/etc/passwd" /etc/passwd
mount --bind "$TASK22_PRIVATE_ROOT/etc/group" /etc/group
mount --bind "$TASK22_PRIVATE_ROOT/varlib" /var/lib
mount --bind "$TASK22_PRIVATE_ROOT/tmp" /tmp
mount --bind "$TASK22_PRIVATE_ROOT/vartmp" /var/tmp
# The genuine producer writes cgroup directory metadata here during setup. This private
# regular filesystem is not claimed as real cgroup/overlay/isolation qualification.
# Ordinary service tests must read the fake mount metadata before rejecting
# its filesystem identity. It remains Root-owned and native-unwritable.
if test "$TASK22_PRIVATE_CASE" = service-package; then chmod 0755 "$TASK22_PRIVATE_ROOT/cgroup"; fi
mount --bind "$TASK22_PRIVATE_ROOT/cgroup" /sys/fs/cgroup
if test "$TASK22_PRIVATE_CASE" = publisher-failure; then
  mount -t tmpfs -o size=64k,mode=0700 tmpfs "$TASK22_PRIVATE_ROOT/run/task22-owner/workspace"
  TASK22_WORKSPACE_TMPFS=1
  chown 1000:1000 "$TASK22_PRIVATE_ROOT/run/task22-owner/workspace"
fi
# The released launcher starts env -i; assign only declared private homes/XDG/temp paths.
# Mount source tmpfs first, then clone the same child into actual /run authority view.
TASK22_RUN_BIND_ATTEMPTED=1
mount --rbind "$TASK22_PRIVATE_ROOT/run" /run
TASK22_RUN_BIND_COMPLETE=1
if test "$TASK22_WORKSPACE_TMPFS" = 1; then TASK22_WORKSPACE_ALIAS_MOUNT=1; fi
export TASK22_PRODUCT TASK22_PRIVATE_ROOT TASK22_PRIVATE_CASE TASK22_SHELL_TEST_BINARY TASK22_SERVICE_TEST_BINARY TASK22_SUBSTRATE_BINARY
if test "$TASK22_PRIVATE_CASE" = preflight || test "$TASK22_PRIVATE_CASE" = preflight-resume || test "$TASK22_PRIVATE_CASE" = preflight-owner || test "$TASK22_PRIVATE_CASE" = preflight-legacy || test "$TASK22_PRIVATE_CASE" = preflight-custody; then
  case "${TASK22_INTEGRATION_TEST_BINARY:?}" in "$TASK22_PRODUCT"/target/debug/deps/consumer_admission_preflight_linux-*) ;; *) exit 91;; esac
  test -x "$TASK22_INTEGRATION_TEST_BINARY"
  export TASK22_INTEGRATION_TEST_BINARY
fi
export TASK22_PRIVATE_NAMESPACE=joined-v1
export PATH=/usr/bin:/bin LANG=C LC_ALL=C PYTHONDONTWRITEBYTECODE=1 PYTHONNOUSERSITE=1
export HOME=/run/task22-root/home USER=root LOGNAME=root
export XDG_CONFIG_HOME="$HOME/.config" XDG_CACHE_HOME="$HOME/.cache" XDG_DATA_HOME="$HOME/.local/share"
export XDG_RUNTIME_DIR=/run/task22-service-runtime TMPDIR=/run/task22-root/tmp
/usr/bin/python3 -I -B - "$TASK22_PRIVATE_ROOT" <<'PY_ALIAS'
import os, sys
for name in ("task22-owner/home", "task22-owner/workspace", "task22-owner/unit-runtime", "task22-owner/tmp", "task22-service-runtime/substrate/worlds"):
    actual = os.stat("/run/" + name)
    measured = os.stat(sys.argv[1] + "/run/" + name)
    if (actual.st_dev, actual.st_ino) != (measured.st_dev, measured.st_ino):
        raise SystemExit(93)
PY_ALIAS
# No inherited provider/Substrate/shim/Python/user-site environment crosses setpriv.
native_command() {
  /usr/bin/setpriv --reuid=1000 --regid=1000 --clear-groups /usr/bin/env -i \
    PATH=/usr/bin:/bin LANG=C LC_ALL=C PYTHONDONTWRITEBYTECODE=1 PYTHONNOUSERSITE=1 \
    HOME=/run/task22-owner/home USER=task22-owner LOGNAME=task22-owner \
    XDG_CONFIG_HOME=/run/task22-owner/home/.config \
    XDG_CACHE_HOME=/run/task22-owner/home/.cache \
    XDG_DATA_HOME=/run/task22-owner/home/.local/share \
    XDG_RUNTIME_DIR=/run/task22-owner/unit-runtime TMPDIR=/run/task22-owner/tmp \
    TASK22_PRODUCT="$TASK22_PRODUCT" TASK22_PRIVATE_ROOT="$TASK22_PRIVATE_ROOT" \
    TASK22_PRIVATE_CASE="$TASK22_PRIVATE_CASE" TASK22_PRIVATE_NAMESPACE=joined-v1 \
    TASK22_SHELL_TEST_BINARY="$TASK22_SHELL_TEST_BINARY" TASK22_SERVICE_TEST_BINARY="$TASK22_SERVICE_TEST_BINARY" \
    TASK22_SUBSTRATE_BINARY="$TASK22_SUBSTRATE_BINARY" "$@"
}
# The added Cargo cases use only this private home and the already cached registry.
# Their actual toolchain/command/source/artifact output is collected by the owner runner.
if test "$TASK22_PRIVATE_CASE" = legacy-cli || test "$TASK22_PRIVATE_CASE" = service-package; then
  mount --bind __TASK22_PRIVATE_HOME__/.cargo/registry "$TASK22_PRIVATE_ROOT/cargo-home/registry"
  TASK22_CARGO_REGISTRY_MOUNT=1
  mount -o remount,bind,ro "$TASK22_PRIVATE_ROOT/cargo-home/registry"
  /usr/bin/ip link set dev lo up
fi
native_cached_cargo() {
  native_command /usr/bin/env \
    PATH=__TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin:__TASK22_PRIVATE_HOME__/.cargo/bin:/usr/bin:/bin \
    CARGO_HOME="$TASK22_PRIVATE_ROOT/cargo-home" RUSTUP_HOME=__TASK22_PRIVATE_HOME__/.rustup RUSTUP_TOOLCHAIN=1.89.0 \
    CARGO_TARGET_DIR="$TASK22_PRODUCT/target" CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 \
    CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_NET_OFFLINE=true RUST_TEST_THREADS=1 \
    RUSTC=__TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/rustc \
    RUSTDOC=__TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/rustdoc \
    CARGO_WORKSPACE_DIR="$TASK22_PRODUCT" "$@"
}
case "$TASK22_PRIVATE_CASE" in
  writer-unit)
    native_command "$TASK22_SHELL_TEST_BINARY" workspace_read_cut_ --nocapture --test-threads=1
    native_command "$TASK22_SHELL_TEST_BINARY" workspace_open_ --nocapture --test-threads=1
    ;;
  legacy-cli)
    case "${TASK22_WORKSPACE_INIT_TEST_BINARY:?}" in "$TASK22_PRODUCT"/target/debug/deps/workspace_init-*) ;; *) exit 91;; esac
    test -x "$TASK22_WORKSPACE_INIT_TEST_BINARY"
    native_cached_cargo __TASK22_PRIVATE_HOME__/.cargo/bin/cargo +1.89.0 build --offline --locked -p substrate --bin substrate --message-format=json
    /usr/bin/sha256sum "$TASK22_SUBSTRATE_BINARY"
    native_cached_cargo "$TASK22_WORKSPACE_INIT_TEST_BINARY" --nocapture --test-threads=1
    /usr/bin/sha256sum "$TASK22_SUBSTRATE_BINARY"
    ;;
  service-package)
    set +e
    native_cached_cargo __TASK22_PRIVATE_HOME__/.cargo/bin/cargo +1.89.0 test --offline --locked -p world-service -- --nocapture
    TASK22_PACKAGE_RESULT=$?
    set -e
    printf 'TASK22_SERVICE_PACKAGE_TEST_EXIT=%s\n' "$TASK22_PACKAGE_RESULT"
    case "$TASK22_PACKAGE_RESULT" in 0|101) ;; *) exit "$TASK22_PACKAGE_RESULT";; esac
    native_cached_cargo __TASK22_PRIVATE_HOME__/.cargo/bin/cargo +1.89.0 build --offline --locked -p world-service --bin world-service --message-format=json
    printf 'TASK22_SERVICE_PRODUCTION_BUILD_EXIT=0\n'
    /usr/bin/sha256sum "$TASK22_PRODUCT/target/debug/world-service"
    exit "$TASK22_PACKAGE_RESULT"
    ;;
  unit)
    native_command "$TASK22_SHELL_TEST_BINARY" workspace_current_ --nocapture --test-threads=1
    native_command "$TASK22_SHELL_TEST_BINARY" workspace_open_ --nocapture --test-threads=1
    native_command "$TASK22_SHELL_TEST_BINARY" workspace_read_cut_ --nocapture --test-threads=1
    for TASK22_PARITY_FILTER in strict_v3_ mixed_start_continuation_and_r0_ancestry start_continuity_registration_and_settlement successor_resume_ successor_attach_ hsa_fork_successor_ typed_current_authority_read_; do
      native_command "$TASK22_SHELL_TEST_BINARY" "$TASK22_PARITY_FILTER" --nocapture --test-threads=1
    done
    ;;
  world)
    for TASK22_WORLD_FILTER in session::tests::persisted_metadata_round_trips_for_recovery session::tests::shared_metadata_round_trips_for_recovery session::tests::shared_binding_state_transitions_persist_and_reject_invalid_edges session::tests::atomic_persist_failure_preserves_prior_metadata_bytes session::tests::exact_bound_world_ session::tests::exact_adopted_join_requires_final_file_resync_before_success; do
      native_command "$TASK22_WORLD_TEST_BINARY" "$TASK22_WORLD_FILTER" --nocapture --test-threads=1
    done
    ;;
  transport-client)
    # Existing client unit tests use only freshly bound127.0.0.1 ephemeral servers.
    # This brings up only loopback inside the newly created empty network namespace.
    /usr/bin/ip link set dev lo up
    native_command "$TASK22_TRANSPORT_CLIENT_TEST_BINARY" --nocapture --test-threads=1
    ;;
  docs)
    # Read only this same task user's existing cached registry, not ambient Cargo config,
    # credentials, account state or another owner's target/index. No cache copy/download.
    mount --bind __TASK22_PRIVATE_HOME__/.cargo/registry "$TASK22_PRIVATE_ROOT/cargo-home/registry"
    TASK22_CARGO_REGISTRY_MOUNT=1
    mount -o remount,bind,ro "$TASK22_PRIVATE_ROOT/cargo-home/registry"
    native_command /usr/bin/env \
      PATH=__TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin \
      CARGO_HOME="$TASK22_PRIVATE_ROOT/cargo-home" CARGO_TARGET_DIR="$TASK22_PRODUCT/target" \
      CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_NET_OFFLINE=true \
      RUSTC=__TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/rustc \
      RUSTDOC=__TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/rustdoc \
      __TASK22_PRIVATE_HOME__/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu/bin/cargo \
      test --offline --locked -p substrate-broker -p transport-api-types -p transport-api-client -p world -p world-service -p shell --doc
    ;;
  *)
    /usr/bin/timeout --signal=TERM --kill-after=5s 240s \
      "$TASK22_SERVICE_TEST_BINARY" \
      service::workspace_current_joined_private_tests::test_workspace_current_joined_private_native_service_publisher_and_lineage \
      --ignored --exact --nocapture --test-threads=1
    ;;
esac
