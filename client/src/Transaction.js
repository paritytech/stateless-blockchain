import React, { useEffect, useState } from 'react';
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
    witness: '',
    transaction: '',
  });

  const { ID, address, witness, transaction } = formState;

  const onChange = (_, data) =>
    setFormState(formState => ({ ...formState, [data.name]: data.value }));

  useEffect(() => {
    console.log(transaction);
  }, [transaction]);

  function createTransaction () {
    const sender = keyring.decodeAddress(accountPair.address, true);
    const receiver = keyring.decodeAddress(address, true);
    const id_num = BigInt(ID);

    const input = { 'pub_key': sender, 'id': id_num };
    const output = { 'pub_key': receiver, 'id': id_num };
    let new_witness = new Uint8Array(witness);
    let tx = { input, output, new_witness}
    setFormState(formState => ({ ...formState, transaction: tx }));
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
              tx: api.tx.stateless && api.tx.stateless.addTransaction
            }}
          />
        </Form.Field>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}
