require('dotenv').config()

const fs = require('fs');
const { ApiPromise, WsProvider  } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring')
const { cryptoWaitReady } = require('@polkadot/util-crypto')

const { PRIVATE_KEY, RPC_URL } = process.env;

async function initPallet(api, alice, owner) {
    console.log('Sending money to the pallet owner', owner.address);
    await initAccount(api, alice, owner.address);

    // Set the current operator (relayer)
    const setOperatorTx = api.tx.zeropool.setOperator(owner.address);

    // Set verification keys
    const transferVk = fs.readFileSync('../keys/transfer_verification_key.bin').toString('hex');
    const transferVkTx = api.tx.zeropool.setTransferVk(`0x${transferVk}`);

    const treeVk = fs.readFileSync('../keys/tree_update_verification_key.bin').toString('hex');
    const treeVkTx = api.tx.zeropool.setTreeVk(`0x${treeVk}`);

    await new Promise(async (res, rej) => {
        const { nonce } = await api.query.system.account(owner.address);

        console.log('Initializing pallet state...')
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

async function sendAndWait(tx, signer) {
    await new Promise(async (res, rej) => {
        await tx.signAndSend(signer, { nonce: -1 }, ({ events = [], status }) => {
            if (status.isFinalized) {
                res(status);
            } else if (status.isInvalid || status.isDropped) {
                rej(status);
            }
        });
    });
}

async function initAccount(api, alice, address) {
    await new Promise(async (res, rej) => {
        console.log('Initializing account with funds', address);
        const { nonce } = await api.query.system.account(alice.address);

        await api.tx.sudo
            .sudo(
                api.tx.balances.setBalance(address, '1000000000000000000000', '0')
            )
            .signAndSend(alice, { nonce }, ({ events = [], status }) => {
                if (status.isFinalized) {
                    console.log('Account initialized', address)
                    res(status);
                } else if (status.isInvalid || status.isDropped) {
                    rej(status);
                }
            });
    });
}

async function test(api, owner) {
    // cradle domain grace animal weapon hill mule frown guess wall reflect shop
    const txDeposit = '0x000000002f016cd77bb066125c9ae64dfe1a2f3358e8faf06bfed10d4b5087abd8793ffe13f72881420c9f802cedb32983d542664fd3c830484e4d1e2524e788ef153c57000000000000000000000000000000000000000000000000000f424027e61b415a867c1cf7e226c29d1970ce48a18b7f34301f75353a344d4d0b0ca81d5164174cb934cae33426e47241edc375bb399f9838e81bd98108039fc5f56923ba05acbbf3b409db5f0f8e545bcabebcec53c1452516b6715e151fd62506950e3dd789d048a7c362e2266499d7461c1507a0aa93dd39c743e4f7772e5816fe2f034b944516077c131f8ea0bdf6fbe97ae6e65da0baa56d0cf9294e54f07cae2b79959664ff00f4499cb9ea8ad47b1bd4b19b493b1ca0abb728c71d4f53f635032c0bb4495e19ffe50595154560151a7f8ef2b91508a1b1983d972d254adc6e251904a807a92db131e324477933b27e79988bc52dc5325eb15e82e30815f815043b313b6460a58496178e565f83b9e1fbbaf2bd2dce63dc9c137f38ec77a3490104d58a18265636d23d3d2fa16c80afba24c07bcdb233a030c5cc8df078698a01671a848afef5c991f9c73cfcd77c66c6f90c1d5b90271312e59e31011b5433048f4b7b309118b33f45fb8c7e21249e719a1b54cbd276289d847aa251af6c12042bbea1939a6897f565e199cbedfa562d30fa687c6f1ed2eec15dceb1a610cc1b4bdec1877f2cb5d5d4e11460d2e49e92f17702ba9ac5ce568c606df2505e9b04d8d0c8b5d718be5b947289c2b7063787cc8ad5c53ffff5b4132533d988ad6c258cfe0e9f8473c95dcb7cb3d1cb61cfb19b5704d405b282ed6b356ca018076d16040b8aa9895daaf0fccc730d39fe9edd2af1b97b4dd030908704f1b10fc939000000d20000000000000000010000005fc6972dbf140056d13058085aa3ea0af4720a3a8f65edb4ed55eb865d7fbf1096913b9cd0abaa806cb66a2a2701076373177db82c6021a589045c3f61e54f06c3f1d5c89253828214644f41949d249eacce3334f58036645f75c6564f835450d0ab85db956325005238437046219c361f289743326a9208be9a99d69604be916aaa0a6f4cec03add59afdd24be47fffcc1eaec8f36f2485d31abc70e2757db502df92f852a05c17947a80024cf59f7c3646c4544608fd1eeefbf6f84aff42ccbf6760fd3b49c8950da545835b5f0a7f5cc091a3bb198620da384790304a73f66bce7aaa0e413c6c930cf5a43545f2d51dbd1c1de15bd6677feb1d58bb6f9984539a31f1d41c82c200c0764711a063c7bb32b07210a9ecd4ea4d7ebd5edfd5ca5919c654e68e';
    const txWithdraw = '0x000000001752268f45c307b5cd98e8e014f79847741a630f13d80aadae6575efcb8011d4237bd19efc59d4cdf65fa4dc2158ee612c79f5219aaf4a3b562a38cc4b7544580000000000800000000000000000000000000000fffffffffff0bdc0028e5a265add45d5dd332594a2a8c7390857b30d07af88c597bb959765e4d8d02db7aee5444c2a5c8c91788b03ae303a497c9d2910c48bedc7df69088b8d872515b4c900a49ecc142b0c9dc5746b000ba252598041d1dc0e9612927b145e566a2d489380ce87d6e406334262b9b8d342f5c1ee5cccebdc22bb3dd9684c33d6862e7e5450244b58f6965ccaaaaeb4404675b64bcb00f5e508c287ff0ff0d00c22057c8e7ff51ced53691dec52abf6d684e31938586309206912f1eb37a5f3e231091f9b781264fe0ba3c2bcb5321b52b6b68ad300664cb9f5ef7320b6b67e45451371f19db8d25cf6abebc4ebae02dbab78c187c9466c00aefb3d2d8ab25ac18f2a6bab1ed748a24cf02aac89cbce321a8a885cad6e9bfcd8dbad2e90ef5797131f51d93a7b91b13728eb01fb77f5ab34bb247741ad2175f5ad5a36ed901764282736fbce202b798b142af110bfb608cee47a9e5fae2ee9d80411d9b0a770f04b2124eac33912dcc9d945e0e6db913b8c56e69468506d01b75c59ccd52861aae01068e76369ac6084b4b7b5f3d808f413bf671bc2c595a11bfdbe21eb2f4e761a10b5a43892aa8790ac51a195ff36e23c925d6cf49b9c7fcdd3aa3d5b9c4159ad22362b648ec2c7b3464a3f51a9221678e44183d66200280e973b8d52ef96ef9e1dd72188e6574534e4fad6e7b1524814bc41f595ee1242bebd3291e6015c6c7110138b28e8146128599bf11acf9af977b53d3daafacbd0fa6c476f84a3a63c95000200fa00000000000000000000000000000000c8950da545835b5f0a7f5cc091a3bb198620da384790304a73f66bce7aaa0e4101000000f22114296822d54170e638e027a8f3f027d8e7f95a62b9cf69cf066f359d65080eb274dcf1bba48eee5ca5741bde6ccb5fb361faea9a207cdd54abea778730179b00104e986f4073a6085b9844c7356421a0fd78c715de19649f250c3d496840512be29753f9862d23072f227c4632b6e602a61270ee22c6d30d740408d53cbd41b43c8eef2708e3df8e8cb0bb11669562384b5c80a6da09c20435cc1fef4c6f1678791927cc30de72e658073a80e9cc43f7b741c563a29321c93b91c44994c1aa0443addaa0';

    console.log('deposit');
    await sendAndWait(api.tx.zeropool.transact(txDeposit), owner);
    console.log('withdraw');
    await sendAndWait(api.tx.zeropool.transact(txWithdraw), owner);
}

async function main() {
    await cryptoWaitReady();

    const wsProvider = new WsProvider(RPC_URL);
    const api = await ApiPromise.create({ provider: wsProvider });

    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const owner = keyring.addFromUri(PRIVATE_KEY);
    const user = keyring.addFromUri('cradle domain grace animal weapon hill mule frown guess wall reflect shop');

    await initPallet(api, alice, owner);
    await initAccount(api, alice, user.address);
    await test(api, owner);

}

main()
    .catch((err) => {
        console.error(err);
        process.exit(1);
    })
    .finally(() => {
        console.log('Done');
        process.exit(0);
    });