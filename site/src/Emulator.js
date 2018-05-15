import React, {Component} from 'react';

class Emulator extends Component {

    constructor(props) {
        super(props);

        this.handleClick = this.handleClick.bind(this);
    }

    handleClick(files) {
        let reader = new FileReader();
        let wasm = this.props.wasm;

        reader.onload = (event) => {
            let arraybuffer = event.target.result;
            let bytes = new Uint8Array(arraybuffer);
            let rom = Array.prototype.slice.call(bytes);
            wasm.then((emulator) => {
                emulator.start(rom);
            });
        };

        reader.readAsArrayBuffer(files[0]);
    }


    render() {
        return (
            <div>
                <canvas id="canvas"/>;
                <input type="file" onChange={(e) => this.handleClick(e.target.files)}/>;
            </div>
        );
    }
}

export default Emulator;