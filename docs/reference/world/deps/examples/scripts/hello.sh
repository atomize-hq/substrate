#!/bin/sh
set -eu

world_deps_root="/var/lib/substrate/world-deps"
world_deps_bin="${world_deps_root}/bin"

mkdir -p "$world_deps_bin"

cat >"${world_deps_bin}/hello" <<'EOF'
#!/bin/sh
echo "hello from world-deps"
EOF

chmod 0755 "${world_deps_bin}/hello"

