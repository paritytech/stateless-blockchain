import React, { useEffect, useState } from 'react';
import { Card, Statistic, Grid } from 'semantic-ui-react';
import { useSubstrate } from './substrate-lib';

export default function State (props) {
  const { api } = useSubstrate();

  // The currently stored value
  const [currentValue, setCurrentValue] = useState('');

  useEffect(() => {
    api.query.stateless.state(value => {
      setCurrentValue(Number(value));
    });
  }, [api.query.stateless]);

  return (
    <Grid.Column>
      <Card>
        <Card.Content textAlign='center'>
          <Statistic label='Accumulator State' />
          <p> {currentValue} </p>
        </Card.Content>
      </Card>
    </Grid.Column>
  );
}
