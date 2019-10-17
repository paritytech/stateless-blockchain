import React, { useState } from 'react';
import { Grid, Form, Input, Button } from 'semantic-ui-react';
import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';
import keyring from '@polkadot/ui-keyring';

export default function Transaction (props) {
  const { api } = useSubstrate();
  const [status, setStatus] = useState(null);
  const { accountPair, wasm } = props;

  const [formState, setFormState] = useState({
    ID: '',
    address: '',
    witness: ''
  });

  const { ID, address, witness } = formState;

  const [transaction, setTransaction] = useState('');

  const onChange = (_, data) =>
    setFormState(formState => ({ ...formState, [data.name]: data.value }));

  function createTransaction () {
    const input = wasm.create_utxo(keyring.decodeAddress(accountPair.address, true), BigInt(ID));
    const output = wasm.create_utxo(keyring.decodeAddress(address, true), BigInt(ID));
    const tx = wasm.create_transaction(input, output, new Uint8Array(witness));
    setTransaction(transaction => { tx });
    alert('Transaction created! Ready to submit to the blockchain.');
  }

  return (
    <Grid.Column>
      <h1>Spend UTXO</h1>
      <Form>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter UTXO ID'
            fluid
            id='input'
            type='text'
            name='ID'
          />
        </Form.Field>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter Output Address'
            fluid
            id='input'
            type='text'
            name='address'
          />
        </Form.Field>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter Witness'
            fluid
            id='input'
            type='text'
            name='witness'
          />
        </Form.Field>
        <Form.Field>
          <Button
            onClick={createTransaction}
            className='ui secondary button'

          >
            Create Transaction
          </Button>
        </Form.Field>
        <Form.Field>
          <TxButton
            accountPair={accountPair}
            label='Submit'
            setStatus={setStatus}
            type='TRANSACTION'
            attrs={{
              params: [transaction],
              tx: api.tx.stateless && api.tx.stateless.add_transaction
            }}
          />
        </Form.Field>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}
