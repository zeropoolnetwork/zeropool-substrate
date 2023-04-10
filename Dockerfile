FROM paritytech/ci-linux:staging as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM node:19
WORKDIR /app
COPY ./js .
RUN yarn
COPY ./docker/startup.sh .
COPY --from=builder /app/target/release/node-template .
ENTRYPOINT ["./startup.sh"]