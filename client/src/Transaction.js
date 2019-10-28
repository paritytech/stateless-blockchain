import React, { useEffect, useState } from 'react';
import { Grid, Form, Input, Button } from 'semantic-ui-react';
import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';
import keyring from '@polkadot/ui-keyring';
import { bnToU8a } from '@polkadot/util';
import { U8a } from '@polkadot/types/codec';

export default function Transaction (props) {
  const { api } = useSubstrate();
  const [status, setStatus] = useState(null);
  const { accountPair } = props;

  const [formState, setFormState] = useState({
    ID: '',
    address: '',
    witness: '',
    transaction: ''
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
    const idNum = BigInt(ID);

    const input = { sender, idNum };
    const output = { receiver, idNum };

    const newWitness = new U8a(bnToU8a(BigInt(witness), 2048, true));
    const tx = { input, output, newWitness };
    setFormState(formState => ({ ...formState, transaction: tx }));
    alert('Transaction created! Ready to submit to the blockchain.');
  }

  return (
    <Grid.Column>
      <h1>Spend Coin</h1>
      <Form>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter Coin ID'
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
