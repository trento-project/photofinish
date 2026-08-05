# SPDX-FileCopyrightText: SUSE LLC
# SPDX-License-Identifier: Apache-2.0

ARG RUST_VERSION=1.96
ARG OS_VER=15.7

# Base build image
FROM registry.suse.com/bci/rust:${RUST_VERSION} AS builder

WORKDIR /home/photofinish/

RUN set -euo pipefail; \
    zypper -n install --no-recommends libopenssl-devel

# Build dependencies first and cache them
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    printf 'fn main() {}\n' > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/photofinish/target \
    cargo build --release

# Copy the actual source code and build the final binary
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/photofinish/target \
    touch src/main.rs && \
    cargo build --release && \
    cp target/release/photofinish /home/photofinish/photofinish

FROM registry.suse.com/bci/bci-base:${OS_VER}

ARG OS_VER

# Copy the built binary from the builder stage
COPY --from=builder /home/photofinish/photofinish /home/photofinish/photofinish

# Install runtime dependencies
RUN set -euo pipefail; \
    zypper -n install --no-recommends ca-certificates libopenssl3

# Cleanup logs and temporary files
RUN set -euo pipefail; zypper -n clean -a; \
    rm -rf {/target,}/var/log/{alternatives.log,lastlog,tallylog,zypper.log,zypp/history,YaST2}; \
    rm -rf {/target,}/run/*; \
    rm -f {/target,}/etc/{shadow-,group-,passwd-,.pwd.lock}; \
    rm -f {/target,}/usr/lib/sysimage/rpm/.rpm.lock; \
    rm -f {/target,}/var/lib/zypp/AnonymousUniqueId; \
    rm -f {/target,}/var/lib/zypp/AutoInstalled; \
    rm -f {/target,}/var/cache/ldconfig/aux-cache

# Define labels according to https://en.opensuse.org/Building_derived_containers
# labelprefix=com.suse.trento
LABEL org.opencontainers.image.authors="https://github.com/trento-project/photofinish/graphs/contributors"
LABEL org.opencontainers.image.title="Photofinish"
LABEL org.opencontainers.image.description="CLI tool to replay events and inject fixtures"
LABEL org.opencontainers.image.documentation="https://github.com/trento-project/photofinish/blob/main/README.adoc"
LABEL org.opencontainers.image.version="devel"
LABEL org.opencontainers.image.url="https://github.com/trento-project/photofinish"
# LABEL org.opencontainers.image.created="" # Set by GHA, no need to set here
LABEL org.opencontainers.image.vendor="SUSE LLC"
LABEL org.opencontainers.image.source="https://github.com/trento-project/photofinish"
LABEL org.opencontainers.image.ref.name="${OS_VER}-devel"
LABEL org.opensuse.reference="registry.suse.com/bci/bci-base:${OS_VER}"
LABEL org.openbuildservice.disturl="https://github.com/trento-project/photofinish/pkgs/container/photofinish"
# endlabelprefix
LABEL org.opencontainers.image.base.name="registry.suse.com/bci/bci-base:${OS_VER}"
LABEL org.opencontainers.image.base.digest="latest"

WORKDIR /data

VOLUME ["/data"]

USER 1001

ENTRYPOINT ["/home/photofinish/photofinish"]
