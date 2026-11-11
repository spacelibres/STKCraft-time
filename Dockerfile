# Build the MCP server and the validation binaries, then ship them on a
# minimal runtime image. The crate has no C dependencies beyond libc, so
# the runtime layer needs nothing but the binaries.

FROM rust:1-bookworm AS build
WORKDIR /src

COPY Cargo.toml Cargo.lock README.md LICENSE ./
COPY src ./src

RUN cargo build --locked --release --features mcp --bins


FROM debian:bookworm-slim AS runtime

ARG VERSION=dev
ARG REVISION=unknown

LABEL org.opencontainers.image.title="stkcraft-time" \
      org.opencontainers.image.description="NAIF/SPICE-aligned astronomical time scales: MCP server and CLI tools" \
      org.opencontainers.image.url="https://www.stkcraft.com" \
      org.opencontainers.image.source="https://github.com/spacelibres/STKCraft-time" \
      org.opencontainers.image.documentation="https://docs.rs/stkcraft-time" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}"

RUN groupadd --gid 10001 stkcraft \
    && useradd --uid 10001 --gid 10001 --no-create-home --shell /usr/sbin/nologin stkcraft

COPY --from=build /src/target/release/stkcraft-mcp /usr/local/bin/
COPY --from=build /src/target/release/sclk-check /usr/local/bin/
COPY --from=build /src/target/release/ut1-check /usr/local/bin/
COPY --from=build /src/target/release/tdb-compare /usr/local/bin/
COPY --from=build /src/target/release/wgc-compare /usr/local/bin/

WORKDIR /data
USER 10001:10001

# `docker run <image>` starts the MCP server; pass a binary name to run one
# of the CLI tools instead, for example `docker run <image> ut1-check file`.
CMD ["/usr/local/bin/stkcraft-mcp"]
