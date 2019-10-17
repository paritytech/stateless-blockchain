import React, { useState, createRef } from 'react';
import { Container, Dimmer, Loader, Grid, Sticky, Divider } from 'semantic-ui-react';

import 'semantic-ui-css/semantic.min.css';
import { SubstrateContextProvider, useSubstrate } from './substrate-lib';
import { DeveloperConsole } from './substrate-lib/components';

import AccountSelector from './AccountSelector';
import Accounts from './Accounts';
import Events from './Events';
import Transaction from './Transaction';
import State from './State';
import Mint from './Mint';
import Witness from './Witness';

const wasmObj = import('accumulator-client');

function Main () {
  const [accountAddress, setAccountAddress] = useState(null);
  const [wasm, setWasm] = useState(null);

  const { apiState, keyring, keyringState } = useSubstrate();
  const accountPair =
    accountAddress &&
    keyringState === 'READY' &&
    keyring.getPair(accountAddress);

  const loader = text => (
    <Dimmer active>
      <Loader size='small'>{text}</Loader>
    </Dimmer>
  );

  if (apiState === 'ERROR') return loader('Error connecting to the blockchain');
  else if (apiState !== 'READY') return loader('Connecting to the blockchain');

  if (keyringState !== 'READY') {
    return loader(
      "Loading accounts (please review any extension's authorization)"
    );
  }

  const contextRef = createRef();

  wasmObj.then(wasm => {
    setWasm(wasm);
  });

  return (
    <div ref={contextRef}>
      <Sticky context={contextRef}>
        <AccountSelector setAccountAddress={setAccountAddress} />
      </Sticky>
      <Container>
        <Grid stackable columns='equal'>
          <Grid.Row>
            <Accounts />
            <Mint accountPair={accountPair} wasm={wasm}/>
          </Grid.Row>
          <Divider />
          <Grid.Row>
            <Transaction accountPair={accountPair} wasm={wasm} />
            <Events />
            <State accountPair={accountPair} />
          </Grid.Row>
          <Divider />
          <Grid.Row>
            <Witness accountPair={accountPair} />
          </Grid.Row>
        </Grid>
        <DeveloperConsole />
      </Container>
    </div>
  );
}

export default function App () {
  return (
    <SubstrateContextProvider>
      <Main />
    </SubstrateContextProvider>
  );
}
