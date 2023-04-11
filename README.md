# ZeroPool substrate pallet

## Run the full test environment
The only two things needed to start the apps is the docker-compose file and the params directory beside it.

This will start an instance of each: substrate node, zeropool-relayer, zeropool-console (web console for testing):
`docker-compose -f docker-compose.full.yml up`

The console will be available on 80 port, the relayer on 3001, and the node on port 9944.
Optionally, provide the `DOCKER_GATEWAY_HOST` variable if running on a remote server.

## Components

### Pallet
`pallets/pallet-zeropool` contains the pallet code.

The example of the pallet usage is in the `runtime/src/lib.rs` file.

### Relayer (zeropool-relayer)
https://github.com/zeropoolnetwork/zeropool-relayer/tree/polkadot

Docker image: https://hub.docker.com/r/voidxnull/zeropool-relayer-polkadot

Implementation of the multi-backend relayer in rust is in progress.

### Client (zeropool-console)
https://github.com/zeropoolnetwork/zeropool-console