import React, { useState } from 'react';
import { u8aToBn } from '@polkadot/util';
import { numberToU8a } from '@polkadot/util';

const Loaded = ({ wasm }) => <button onClick={
  console.log(BigInt(u8aToBn(wasm.hash_to_prime([7, 10]))))
  }>
    Click me
  </button>;

const Unloaded = ({ loading, loadWasm }) => {
  return loading ? (
    <div>Loading...</div>
  ) : (
    <button onClick={loadWasm}>Load library</button>
  );
};

const App = () => {
  const [loading, setLoading] = useState(false);
  const [wasm, setWasm] = useState(null);

  const loadWasm = async () => {
    try {
      setLoading(true);
      const wasm = await import('accumulator');
      setWasm(wasm);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="App">
      <header className="App-header">
        {wasm ? (
          <Loaded wasm={wasm} />
        ) : (
          <Unloaded loading={loading} loadWasm={loadWasm} />
        )}
      </header>
    </div>
  );
};

export default App;
