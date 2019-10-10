import React, { useState } from 'react';
import { Grid, Form, Input } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

export default function Transaction (props) {
  const { api } = useSubstrate();
  const [status, setStatus] = useState(null);
  const { accountPair } = props;

  const [formState, setFormState] = useState({
    input: '',
    output: '',
    witness: '',
    proof: ''
  });
  const { input, output, witness, proof } = formState;

  const onChange = (_, data) =>
    setFormState({ [data.name]: data.state });

  return (
    <Grid.Column>
      <h1>Construct and Submit Transaction</h1>
      <Form>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter Input UTXO'
            fluid
            state='input'
            type='text'
            name='input'
          />
        </Form.Field>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter Output UTXO'
            fluid
            id='input'
            type='text'
            name='output'
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
          <Input
            onChange={onChange}
            label='Enter Proof'
            fluid
            id='input'
            type='text'
            name='proof'
          />
        </Form.Field>
        <Form.Field>
          <TxButton
            accountPair={accountPair}
            label='Submit Transaction'
            setStatus={setStatus}
            type='TRANSACTION'
            attrs={{
              params: [{ input, output, witness, proof }],
              tx: api.tx.stateless && api.tx.stateless.add_transaction
            }}
          />
        </Form.Field>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}
