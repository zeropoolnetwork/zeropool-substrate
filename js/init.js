require('dotenv').config()
const fs = require('fs');
const { ApiPromise, WsProvider  } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring')
const { cryptoWaitReady } = require('@polkadot/util-crypto')

const { PRIVATE_KEY, RPC_URL } = process.env;

async function initPalet() {
    await cryptoWaitReady();
    const wsProvider = new WsProvider(RPC_URL);
    const api = await ApiPromise.create({ provider: wsProvider });

    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const owner = keyring.addFromUri(PRIVATE_KEY);

    const { nonce } = await api.query.system.account(alice.address);
    await api.tx.sudo
        .sudo(
            api.tx.balances.setBalance(owner.address, '1000000000000000000000', '0')
        )
        .signAndSend(alice, { nonce });

    // Set the current operator (relayer)
    const setOperatorTx = api.tx.zeropool.setOperator(owner.address);

    // Set verification keys
    const transferVk = fs.readFileSync('../keys/transfer_verification_key.bin').toString('hex');
    const transferVkTx = api.tx.zeropool.setTransferVk(`0x${transferVk}`);

    const treeVk = fs.readFileSync('../keys/tree_update_verification_key.bin').toString('hex');
    const treeVkTx = api.tx.zeropool.setTreeVk(`0x${treeVk}`);

    await new Promise(async (res, rej) => {
        const { nonce } = await api.query.system.account(owner.address);

        await api.tx.utility
            .batch([setOperatorTx, transferVkTx, treeVkTx])
            .signAndSend(owner, { nonce }, ({ events = [], status }) => {
                console.log('Transaction status:', status.type);

                if (status.isInBlock) {
                    console.log('Included at block hash', status.asInBlock.toHex());
                    console.log('Events:');

                    events.forEach(({ event: { data, method, section }, phase }) => {
                        console.log('\t', phase.toString(), `: ${section}.${method}`, data.toString());
                    });
                } else if (status.isFinalized) {
                    console.log('Finalized block hash', status.asFinalized.toHex());
                    res(status);
                } else if (status.isInvalid || status.isDropped) {
                    rej(status);
                }
            });
    });
}

async function getEvents() {

}

async function main() {
    await initPalet();


}

main()
    .finally(() => {
        console.log('Done');
        process.exit(0);
    })
    .catch((err) => {
        console.error(err);
        process.exit(1);
    })