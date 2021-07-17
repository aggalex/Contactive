FROM rustlang/rust:nightly as planner
WORKDIR contactive
# We only pay the installation cost once, 
# it will be cached from the second build onwards
# To ensure a reproducible build consider pinning 
# the cargo-chef version with `--version X.X.X`
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rustlang/rust:nightly as cacher
WORKDIR contactive
RUN cargo install cargo-chef
COPY --from=planner /contactive/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rustlang/rust:nightly as builder
WORKDIR contactive
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /contactive/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin contactive

FROM postgres:latest as runtime

EXPOSE 8000

WORKDIR /contactive
COPY --from=builder /contactive/target/release/contactive /usr/local/bin
COPY --from=planner /contactive/wait.sh /usr/local/bin
COPY --from=planner /contactive/env .

RUN [ -f /usr/local/bin/contactive ]

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

EXPOSE 8000

CMD [ "wait.sh", "postgresql", "/usr/local/bin/contactive" ]
