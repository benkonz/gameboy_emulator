import React, {Component} from 'react';
import Emulator from './Emulator';

class App extends Component {
    render() {
        return (
            <Emulator wasm={this.props.config.wasm}/>
        );
    }
}

export default App;