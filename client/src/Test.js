import React, { useState } from 'react';

const Loaded = ({ wasm }) => <button onClick={wasm.greet}>Click me</button>;

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
      const wasm = await import('accumulator-client');
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