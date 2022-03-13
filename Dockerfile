FROM paritytech/ci-linux:staging as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM paritytech/ci-linux:staging
COPY ./js .
COPY ./docker/startup.sh .
COPY --from=build /app/target/release/node-template .
ENTRYPOINT ["./startup.sh"]