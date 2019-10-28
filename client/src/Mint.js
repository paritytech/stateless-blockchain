import React, { useState } from 'react';
import { Grid, Form, Input, Button } from 'semantic-ui-react';
import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';
import { u8aToBn } from '@polkadot/util';
import keyring from '@polkadot/ui-keyring';

export default function Mint (props) {
  const { api } = useSubstrate();
  const [status, setStatus] = useState(null);
  const { accountPair, wasm } = props;

  const [UTXO, setUTXO] = useState({
    ID: '',
    elem: ''
  });

  const { ID, elem } = UTXO;

  const onChange = (_, data) =>
    setUTXO(UTXO => ({ ...UTXO, [data.name]: data.value }));

  function createUTXO () {
    const utxo = wasm.get_utxo_elem(keyring.decodeAddress(accountPair.address, true), BigInt(ID));
    const hash = BigInt(u8aToBn(wasm.hash_to_prime(new Uint8Array(utxo))));
    setUTXO(UTXO => ({ ...UTXO, elem: hash }));
    alert('The value of the coin is: ' + hash);
  }

  return (
    <Grid.Column>
      <h1>Mint Coin</h1>
      <Form>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter ID for Coin'
            fluid
            id='input'
            type='text'
            name='ID'
          />
        </Form.Field>
        <Form.Field>
          <Button
            onClick={createUTXO}
            className='ui secondary button'

          >
            Get Hash
          </Button>
        </Form.Field>
        <Form.Field>
          <TxButton
            accountPair={accountPair}
            label='Add Coin to Accumulator'
            setStatus={setStatus}
            type='TRANSACTION'
            attrs={{
              params: [elem],
              tx: api.tx.stateless && api.tx.stateless.mint
            }}
          />
        </Form.Field>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}
