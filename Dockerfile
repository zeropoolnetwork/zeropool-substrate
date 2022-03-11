FROM paritytech/ci-linux:staging as build

RUN apt-get update && \
    apt-get -y install curl

# create a new empty shell project
#RUN USER=root cargo new --bin node-template
WORKDIR /app

## copy over your manifests
#COPY ./Cargo.lock ./Cargo.lock
#COPY ./Cargo.toml ./Cargo.toml
#
## this build step will cache your dependencies
#RUN cargo build --release
#RUN rm src/*.rs

# copy your source tree
COPY . .

# build for release
#RUN rm ./target/release/deps/node-template*
RUN cargo build --release

# our final base
FROM paritytech/ci-linux:staging

# copy the build artifact from the build stage
COPY --from=build /app/target/release/node-template .

# set the startup command to run your binary
CMD ["./node-template --dev --ws-external"]