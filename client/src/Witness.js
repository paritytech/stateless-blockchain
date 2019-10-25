import React, { useState } from 'react';
import { Grid, Form, Input, Button } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

export default function Witness (props) {
  const { api } = useSubstrate();
  const [status, setStatus] = useState(null);
  const { accountPair } = props;

  const onChange = (_, data) =>
      ({ });

  return (
    <Grid.Row columns={2}>
      <Grid.Column>
        <h1>Receive Witness</h1>
        <Form>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter Element'
              fluid
              id='input'
              type='text'
              name='elem'
            />
          </Form.Field>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter State before Addition'
              fluid
              state='input'
              type='text'
              name='old_state'
            />
          </Form.Field>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter Added Elements'
              fluid
              id='input'
              type='text'
              name='added'
            />
          </Form.Field>
          <div style={{ overflowWrap: 'break-word' }}>{status}</div>
        </Form>
      </Grid.Column>
      <Grid.Column>
        <h1>Update Witness</h1>
        <Form>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter Element'
              fluid
              state='input'
              type='text'
              name='elem'
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
              label='Enter Added Elements'
              fluid
              id='input'
              type='text'
              name='added'
            />
          </Form.Field>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter Deleted Elements'
              fluid
              id='input'
              type='text'
              name='deleted'
            />
          </Form.Field>
          <div style={{ overflowWrap: 'break-word' }}>{status}</div>
        </Form>
      </Grid.Column>
    </Grid.Row>
  );
}
