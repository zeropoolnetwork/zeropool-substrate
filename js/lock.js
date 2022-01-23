const { ApiPromise, ApiRx, WsProvider } = require('@polkadot/api');
const { mnemonicGenerate, mnemonicValidate, cryptoWaitReady  } = require('@polkadot/util-crypto')
const { Keyring } = require('@polkadot/keyring')
const { stringToU8a } = require('@polkadot/util');
const { of, tap, first, from, switchMap, catchError } = require('rxjs');

const keyring = new Keyring({type: 'sr25519'})  // init key store
const AMOUNT = 100000
let ALICE, BOB, CHARLIE, DAVE, EVE, FERDIE, FROM, TO


const initAccounts = () => {
  // keys defined here: @polkadot/keyring/testing.js
  ALICE = keyring.addFromUri('//Alice', { name: 'Alice default' })
  BOB = keyring.addFromUri('//Bob', { name: 'Alice default' })
  DAVE = keyring.addFromUri('//Dave', { name: 'Alice default' })
  EVE = keyring.addFromUri('//Eve', { name: 'Alice default' })
  FERDIE = keyring.addFromUri('//Ferdie', { name: 'Alice default' })
}

const lock = (api$) =>
  api$.pipe(
    switchMap(api =>
      api.query.system.account(FROM.address).pipe(
        first(),
        switchMap(([nonce]) =>
          api.tx.zeropool
            .lock(AMOUNT)
            .sign(FROM, { nonce: nonce[1].words[0] })
            .send()
        ),
      )
    )
  )

const main = () => {
  initAccounts()

  FROM = ALICE
  TO = BOB
  
  new ApiRx({ provider: new WsProvider('ws://127.0.0.1:9944') })
    .isReady
    .pipe(
      lock,
      // release,
    )
    .subscribe((result) => {
      if(result instanceof Error) {
        throw result
      }
      
      if (result.status.isInBlock) {
        console.log('Successful lock of ' + AMOUNT + ' with hash ' + result.status.asInBlock.toHex());
      } else {
        console.log('Status of transfer: ' + result.status.type)

        if (result.status.isFinalized) {
          console.log('Finalized block hash: ' + result.status.asFinalized.toHex())
          process.exit()
        }
      }
    
      result.events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString());
      })
    })
}

cryptoWaitReady()
  .then(main)
  .catch(console.error)
