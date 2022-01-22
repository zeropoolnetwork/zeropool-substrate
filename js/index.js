const { ApiPromise, WsProvider } = require('@polkadot/api');
const { mnemonicGenerate, mnemonicValidate } = require('@polkadot/util-crypto')
const { Keyring } = require('@polkadot/keyring')

const keyring = new Keyring({type: 'sr25519'})  // init key store
const endpoint = 'ws://127.0.0.1:9944'

const createAccount = (_mnemonic) => {
  const mnemonic = _mnemonic && mnemonicValidate(_mnemonic) 
    ? _mnemonic 
    : mnemonicGenerate();

  return { account: keyring.addFromMnemonic(mnemonic), mnemonic }
}

const connect = async () => {
  const api = new ApiPromise({ provider: new WsProvider(endpoint) })
  debugger
  return api
}

const main = async (api) => {
  console.log(`Our client is connected: ${api?.isConnected}`);

  const mnemonic = 'cruel leader remember night skill clump question focus nurse neck battle federal';
  const { account: acc1 } = createAccount(mnemonic);
  const balance = await api.derive.balances.all(acc1.address);
  const available = balance.availableBalance.toNumber();
  const dots = available / (10 ** api.registry.chainDecimals);
  const print = dots.toFixed(4);

  console.log(`Address ${medium1.address} has ${print} DOT`);
};

connect()
  .then(main)
  .catch((err) => console.error(err))
  .finally(() => process.exit());