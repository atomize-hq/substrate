# Dev container for running Codex and full Linux testing for substrate
# Base: official Rust image (Debian trixie) with Rust 1.89
FROM rust:1.89

SHELL ["/bin/bash", "-lc"]

# Avoid tzdata prompts, speed up apt
ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_TERM_COLOR=always \
    RUST_LOG=info \
    HOME=/root \
    CODEX_HOME=/root/.codex \
    CODEX_UNSAFE_ALLOW_NO_SANDBOX=1

# Core tooling and kernel/net utils required for tests
# - build: clang, llvm, cmake, pkg-config, libssl-dev
# - net: nftables, conntrack, iproute2, iptables, iputils-ping, net-tools, tcpdump
# - sys: kmod, procps, util-linux, jq, ripgrep, curl, sudo, lsof, strace
# - optional: fuse-overlayfs, uidmap (for rootless/userns fallback)
RUN set -euo pipefail \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates curl gnupg \
      build-essential cmake pkg-config \
      clang llvm-dev \
      libssl-dev libseccomp-dev \
      git jq ripgrep \
      iproute2 iptables nftables conntrack \
      iputils-ping net-tools tcpdump \
      kmod procps util-linux sudo \
      lsof strace \
      fuse-overlayfs uidmap \
      python3 python3-pip \
      nodejs npm \
      dnsutils ipset \
      less unzip zsh \
      fd-find bat \
  && update-ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Provide convenient names for Debian packaging quirks
RUN set -euo pipefail \
  && if [[ -x /usr/bin/fdfind && ! -e /usr/local/bin/fd ]]; then ln -s /usr/bin/fdfind /usr/local/bin/fd; fi \
  && if [[ -x /usr/bin/batcat && ! -e /usr/local/bin/bat ]]; then ln -s /usr/bin/batcat /usr/local/bin/bat; fi

# Create workspace dir and helpful defaults
WORKDIR /src

# Optional: create a non-root user "codex" with passwordless sudo
RUN useradd -m -u 1000 -s /bin/bash codex \
  && echo 'codex ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/99-codex \
  && chmod 0440 /etc/sudoers.d/99-codex

# Install Codex CLI (default via npm).
# Default: npm i -g @openai/codex
# Build-time options (choose one if needed):
#   --build-arg CODEX_NPM_PKG=@openai/codex
#   --build-arg CODEX_PIP_PKG=codex-cli
#   --build-arg CODEX_GIT_URL=https://github.com/openai/codex.git
ARG CODEX_NPM_PKG=""
ARG CODEX_PIP_PKG=""
ARG CODEX_GIT_URL=""

RUN set -euo pipefail \
 && if [[ -n "${CODEX_NPM_PKG}" ]]; then \
      echo "[docker] Installing Codex via npm: ${CODEX_NPM_PKG}" && npm install -g "${CODEX_NPM_PKG}"; \
    else \
      echo "[docker] Installing Codex via npm (default: @openai/codex)" \
      && npm install -g @openai/codex; \
    fi \
 && if [[ -n "${CODEX_PIP_PKG}" ]]; then \
      echo "[docker] Installing Codex via pip: ${CODEX_PIP_PKG}" && python3 -m pip install --upgrade pip wheel && pip install "${CODEX_PIP_PKG}"; \
    else echo "[docker] Skipping pip Codex install"; fi \
 && if [[ -n "${CODEX_GIT_URL}" ]]; then \
      echo "[docker] Installing Codex from git: ${CODEX_GIT_URL}" && git clone "${CODEX_GIT_URL}" /opt/codex && \
      if [[ -x /opt/codex/install.sh ]]; then /opt/codex/install.sh || true; fi; \
    else echo "[docker] Skipping git Codex install"; fi

# Entry script: mounts and sysctls needed for tests when running with --privileged
COPY scripts/dev-entrypoint.sh /usr/local/bin/dev-entrypoint.sh
RUN chmod +x /usr/local/bin/dev-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/dev-entrypoint.sh"]
CMD ["bash"]
