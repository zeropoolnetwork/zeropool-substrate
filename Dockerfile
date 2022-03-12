FROM paritytech/ci-linux:staging as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM paritytech/ci-linux:staging
COPY --from=build /app/target/release/node-template .
ENTRYPOINT ["./node-template", "--dev", "--ws-external"]