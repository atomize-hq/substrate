# Running Privileged Tests (Linux)

Some world isolation and netfilter tests require elevated privileges (CAP_SYS_ADMIN) and kernel features (overlayfs, nftables).

## Option A: Local host (root)

```
sudo -E apt-get update && sudo -E apt-get install -y nftables conntrack
sudo -E RUST_LOG=info cargo test -p world -- --nocapture || true
```

## Option B: Docker container (recommended)

```
docker run --rm -it --privileged \
  -v "$PWD":/src -w /src \
  rust:1.80 bash -lc 'apt-get update && apt-get install -y nftables conntrack && cargo test -p world -- --nocapture'
```

Notes
- Tests skip or degrade gracefully when capabilities are missing.
- Never run privileged tests on untrusted machines.

