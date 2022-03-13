const { first, switchMap, take, combineLatest, tap, of } = require('rxjs');
const { ApiRx, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady  } = require('@polkadot/util-crypto')
const { Keyring } = require('@polkadot/keyring')
const { BN } = require('bn.js');

let ALICE, BOB, CHARLIE, DAVE, EVE, FERDIE, FROM, TO, AMOUNT, FACTOR

const keyring = new Keyring({type: 'sr25519'})  // init key store
const initAccounts = () => {
  // keys defined here: @polkadot/keyring/testing.js
  ALICE = keyring.addFromUri('//Alice', { name: 'Alice default' })
  BOB = keyring.addFromUri('//Bob', { name: 'Alice default' })
  DAVE = keyring.addFromUri('//Dave', { name: 'Alice default' })
  EVE = keyring.addFromUri('//Eve', { name: 'Alice default' })
  FERDIE = keyring.addFromUri('//Ferdie', { name: 'Alice default' })
}

const initAmounts = (api$) =>
  api$.pipe(
    tap((api) => {
      FACTOR = new BN(10).pow(new BN(api.registry.chainDecimals));
      AMOUNT = new BN(15).mul(FACTOR);
    })
  )

const balance = async (api, address) => {
  const balance = await api.derive.balances.all(address).pipe(take(1)).toPromise()
  const available = balance.availableBalance
  
  return available / FACTOR
}

const lock = (api$) =>
  api$.pipe(
    switchMap(api =>
      api.query.system.account(FROM.address).pipe(
        first(),
        switchMap(([nonce]) => combineLatest([
          api.tx.zeropool
            .lock(AMOUNT)
            .sign(FROM, { nonce: nonce[1].words[0] })
            .send(),
          of(api),
        ])),
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
      initAmounts,
      lock,
    )
    .subscribe(([result, api]) => {
      if(result instanceof Error) {
        throw result
      }
      
      if (result.status.isInBlock) {
        console.log('Successful lock of ' + AMOUNT / FACTOR + ' DOT with hash ' + result.status.asInBlock.toHex());
      } else {
        console.log('Status of transfer: ' + result.status.type)

        if (result.status.isFinalized) {
          console.log('Finalized block hash: ' + result.status.asFinalized.toHex())
          balance(api, FROM.address).then(print => {
            console.log(`Address ${FROM.address} (${FROM.meta.name}) has ${print} DOT`)
            process.exit()
          })
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
