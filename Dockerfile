FROM paritytech/ci-linux:staging as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM node:16

WORKDIR /app
COPY ./js .
RUN yarn
COPY ./docker/startup.sh .
COPY --from=build /app/target/release/node-template .
ENTRYPOINT ["./startup.sh"]