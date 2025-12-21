FROM --platform=${BUILDPLATFORM} debian:bookworm-slim
WORKDIR /build

RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential=12.9 ca-certificates=20230311+deb12u1 curl=7.88.1-10+deb12u14 && \
    rm -rf /var/lib/apt/lists/*

RUN mkdir /app

COPY container_src/ .

RUN ./rustowl/scripts/build/toolchain cargo install --locked --path rustowl --root /app && rm -rf rustowl/target/

RUN ./rustowl/scripts/build/toolchain cargo install --locked --path . --root /app && rm -rf target/

WORKDIR /app
ENV PATH=/app/bin:$PATH
EXPOSE 3000
ENTRYPOINT ["rustowl-container"]
