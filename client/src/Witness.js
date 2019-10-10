import React, { useState } from 'react';
import { Grid, Form, Input } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

export default function Witness (props) {
  const { api } = useSubstrate();
  const [status, setStatus] = useState(null);
  const { accountPair } = props;

  const onChange = (_, data) =>
      ({ });

  return (
    <Grid.Column>
      <h1>Receive or Update Witness</h1>
      <Form>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Receive Witness for New UTXO'
            fluid
            state='input'
            type='text'
            name='new'
          />
        </Form.Field>
        <Form.Field>
          <Input
            onChange={onChange}
            label='Update Witness'
            fluid
            id='input'
            type='text'
            name='update'
          />
        </Form.Field>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}
