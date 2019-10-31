import React, { useEffect, useState } from 'react';
import { Card, Statistic, Grid } from 'semantic-ui-react';
import { useSubstrate } from './substrate-lib';

import { u8aToBn } from '@polkadot/util';

export default function State (props) {
  const { api } = useSubstrate();

  // The currently stored value
  const [currentValue, setCurrentValue] = useState('');

  useEffect(() => {
    api.query.stateless.state(value => {
      setCurrentValue(Number(BigInt(u8aToBn(value))));
    });
  }, [api.query.stateless]);

  return (
    <Grid.Column>
      <h3> Accumulator Value: {currentValue} </h3>
    </Grid.Column>
  );
}
