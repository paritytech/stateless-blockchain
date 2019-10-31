import React, { useState } from 'react';
import { Grid, Form, Input, Button } from 'semantic-ui-react';
import { u8aToBn, bnToU8a } from '@polkadot/util';

export default function Witness (props) {
  const [status, setStatus] = useState(null);
  const { wasm } = props;

  const [formState, setFormState] = useState({
    elem: '',
    oldState: '',
    newState: '',
    added: '',
    deleted: '',
    witness: ''
  });

  const { elem, oldState, newState, added, deleted, witness } = formState;

  const onChange = (_, data) =>
    setFormState(formState => ({ ...formState, [data.name]: data.value }));

  function getWitness () {
    const newWitness = BigInt(u8aToBn(wasm.get_witness(bnToU8a(oldState), bnToU8a(added), bnToU8a(elem))));
    alert('The witness is: ' + newWitness);
  }

  function updateWitness () {
    const updatedWitness = BigInt(u8aToBn(wasm.update_witness(bnToU8a(elem), bnToU8a(witness), bnToU8a(newState), bnToU8a(added), bnToU8a(deleted))));
    alert('The witness is: ' + updatedWitness);
  }

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
              name='oldState'
            />
          </Form.Field>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter Product of Added Elements'
              fluid
              id='input'
              type='text'
              name='added'
            />
          </Form.Field>
          <Form.Field>
            <Button
              onClick={getWitness}
              className='ui secondary button'

            >
              Get Witness
            </Button>
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
              label='Enter Product of Added Elements'
              fluid
              id='input'
              type='text'
              name='added'
            />
          </Form.Field>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter Product of Deleted Elements'
              fluid
              id='input'
              type='text'
              name='deleted'
            />
          </Form.Field>
          <Form.Field>
            <Input
              onChange={onChange}
              label='Enter the Current State'
              fluid
              id='input'
              type='text'
              name='newState'
            />
          </Form.Field>
          <Form.Field>
            <Button
              onClick={updateWitness}
              className='ui secondary button'

            >
              Get Witness
            </Button>
          </Form.Field>
          <div style={{ overflowWrap: 'break-word' }}>{status}</div>
        </Form>
      </Grid.Column>
    </Grid.Row>
  );
}
