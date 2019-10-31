import React, { useState, createRef } from 'react';
import { Container, Dimmer, Loader, Grid, Sticky, Divider, List } from 'semantic-ui-react';
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
            <h1> Instructions </h1>
            <List ordered>
              <List.Item>
                Mint a coin.
                <List.List>
                  <List.Item>Enter a 64 bit number as the coin ID.</List.Item>
                  <List.Item>Get the hash prime representation of the coin.</List.Item>
                  <List.Item>The witness associated with this coin is the current accumulator value.</List.Item>
                  <List.Item>Add the coin to the accumulator.</List.Item>
                </List.List>
              </List.Item>
              <List.Item>
                Send a coin.
                <List.List>
                  <List.Item>Enter the ID of the coin. Since we are dealing with "non-fungible" tokens this
                  must be the same as before.</List.Item>
                  <List.Item>Enter an output address from the given accounts table. You cannot send a coin to yourself.</List.Item>
                  <List.Item>Enter the witness for the coin.</List.Item>
                  <List.Item>Create a transaction and submit it to the blockchain.</List.Item>
                </List.List>
              </List.Item>
              <List.Item>
                Receive or update a witness.
                <List.List>
                  <List.Item>To receive the witness for a coin after either minting or receiving it, enter
                  the hash representation of the coin, the state before the coin was added, and the product
                  of any other added elements (you should be able to read this value from the events log).</List.Item>
                  <List.Item>To update a witness, enter the hash representation of the coin, the current witness,
                  the product of any added elements, the product of any deleted elements, and the current state of the
                  accumulator. Again, you should be able to read these values from the events log.
                  </List.Item>
                </List.List>
              </List.Item>
            </List>
          </Grid.Row>
          <Divider />
          <Grid.Row>
            <State accountPair={accountPair} />
          </Grid.Row>
          <Divider />
          <Grid.Row>
            <Accounts />
            <Events />
          </Grid.Row>
          <Divider />
          <Grid.Row>
            <Mint accountPair={accountPair} wasm={wasm}/>
            <Transaction accountPair={accountPair} />
          </Grid.Row>
          <Divider />
          <Grid.Row>
            <Witness wasm={wasm} />
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
