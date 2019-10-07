import React, { useEffect, useState } from 'react';
import { Grid } from 'semantic-ui-react';
import { useSubstrate } from './substrate-lib';

import { u8aToHex } from '@polkadot/util';

export default function State (props) {
  const { api } = useSubstrate();

  // The currently stored value
  const [currentValue, setCurrentValue] = useState('');

  useEffect(() => {
    api.query.stateless.state(value => {
      setCurrentValue(u8aToHex(new Uint8Array(value)));
    });
  }, [api.query.stateless]);

  return (
    <Grid.Column>
      <h1> Accumulator State</h1>
      <p> {currentValue} </p>
    </Grid.Column>
  );
}
