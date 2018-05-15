import wasm from '../../lib/src/lib.rs'
import index from './index.html'
import React from 'react'
import ReactDOM from 'react-dom'
import App from './App.js'


const config = {
    wasm: wasm
};

ReactDOM.render(<App config={config}/>, document.getElementById("root"));