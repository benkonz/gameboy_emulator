const path = require('path');

module.exports = {
    node: {
        fs: 'empty'
    },
    entry: './src/index.js',
    output: {
        filename: 'index.js',
        path: path.resolve(__dirname, 'dist')
    },
    module: {
        rules: [
            {
                test: /\.html/,
                loader: 'file-loader?name=[name].[ext]',
            },
            {
                test: /\.rs$/,
                loader: 'rust-native-wasm-loader',
                options: {
                    cargoWeb: true
                }
            },
            {
                test: /\.js/,
                loader: 'babel-loader',
                exclude: /node_modules/,
                options: {
                    presets: ['env', 'react']
                }
            }
        ]
    },
    devtool: 'source-map'
};