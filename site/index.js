import wasm from '../lib/src/lib.rs'
import index from './index.html'

wasm.then(module => {
    document.getElementById("file_selector")
        .addEventListener("change", function () {
            let reader = new FileReader();
            reader.onload = function () {
                let arraybuffer = this.result;
                let bytes = new Uint8Array(arraybuffer);
                let rom = Array.prototype.slice.call(bytes);
                module.start(rom);
            };

            reader.readAsArrayBuffer(this.files[0]);
        });
});