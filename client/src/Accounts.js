import React from 'react';
import { Table, Grid } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';

export default function Accounts (props) {
  const { keyring } = useSubstrate();
  const accounts = keyring.getPairs();

  return (
    <Grid.Column>
      <h1>Accounts</h1>
      <Table celled striped size='small'>
        <Table.Body>{accounts.map(account =>
          <Table.Row key={account.address}>
            <Table.Cell textAlign='right'>{account.meta.name}</Table.Cell>
            <Table.Cell>{account.address}</Table.Cell>
          </Table.Row>
        )}
        </Table.Body>
      </Table>
    </Grid.Column>
  );
}
