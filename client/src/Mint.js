import React, { useState } from 'react';
import { Grid, Form, Input, Button } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

export default function Mint (props) {
  const { api } = useSubstrate();
  const [status, setStatus] = useState(null);
  const { accountPair } = props;

  const [formState, setFormState] = useState({
    pubKey: '',
    ID: ''
  });
  const { pubKey, ID } = formState;

  const onChange = (_, data) => setFormState({ [data.name]: data.state });
  return (
    <Grid.Column>
      <h1>Mint UTXO</h1>
      <Form>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter Public Key'
            fluid
            state='input'
            type='text'
            name='pubKey'
          />
        </Form.Field>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Enter ID for UTXO'
            fluid
            id='input'
            type='text'
            name='ID'
          />
        </Form.Field>
        <Form.Field>
          <Button
            onClick={() => setStatus('Hash created here.')}
            className='ui secondary button'
          >
            Get Hash
          </Button>
        </Form.Field>
        <Form.Field>
          <TxButton
            accountPair={accountPair}
            label='Add UTXO to Accumulator'
            setStatus={setStatus}
            type='TRANSACTION'
            attrs={{
              params: [pubKey, ID],
              tx: api.tx.stateless && api.tx.stateless.mint
            }}
          />
        </Form.Field>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}
