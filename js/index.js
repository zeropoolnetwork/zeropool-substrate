const { ApiPromise, WsProvider } = require('@polkadot/api');
const { mnemonicGenerate, mnemonicValidate, cryptoWaitReady  } = require('@polkadot/util-crypto')
const { Keyring } = require('@polkadot/keyring')

const keyring = new Keyring({type: 'sr25519'})  // init key store
const MNEMONIC = 'cruel leader remember night skill clump question focus nurse neck battle federal'
const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'
const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty'
const AMOUNT = 100000

const createAccount = (_mnemonic) => {
  const mnemonic = _mnemonic && mnemonicValidate(_mnemonic) 
    ? _mnemonic 
    : mnemonicGenerate();
  const account = keyring.addFromUri(mnemonic);
  
  return { account, mnemonic }
}

const balance = async (api, address) => {
  const balance = await api.derive.balances.all(address);
  const available = balance.availableBalance.toString();
  const dots = available / (10 ** api.registry.chainDecimals);
  
  return dots.toFixed(4);
}

const connect = async () => {
  const wsProvider = new WsProvider('ws://127.0.0.1:9944');
  const api = new ApiPromise({ provider: wsProvider });
  
  return api.isReady;
}

const lock = async (api, from, pool, amount) => {
  const transfer = api.tx.lock(pool, amount)

  await transfer.signAndSend(from, ({ events = [], status }) => {
    if (status.isInBlock) {
      console.log('Successful lock of ' + amount + ' with hash ' + status.asInBlock.toHex());
    } else {
      console.log('Status of transfer: ' + status.type);
    }
  
    events.forEach(({ phase, event: { data, method, section } }) => {
      console.log(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString());
    });
  });
}

const main = async (api) => {
  console.log(`Our client is connected: ${api?.isConnected}`);

  const { account: acc1 } = createAccount(MNEMONIC);
  

  console.log(`Address ${acc1.address} has ${await balance(api, acc1.address)} DOT`);
  console.log(`Address ${ALICE} has ${await balance(api, ALICE)} DOT`);
  console.log(`Locking ${AMOUNT} DOT on ${ALICE}...`);

  await lock(api, ALICE, acc1.address, AMOUNT);


};

cryptoWaitReady().then(() => {
  connect()
    .then(main)
    .catch((err) => console.error(err))
    .finally(() => process.exit())
})