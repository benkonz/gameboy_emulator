module.exports = {
    entry: ,
    output: {
      filename: 'bundle.js',
      path: __dirname + '/build',
    },
    module: {
      rules: [
        {
          test: /\.rs$/,
          use: {
            loader: 'rust-wasm-loader',
            options: {
              // The path to the webpack output relative to the project root
              path: 'build'
            }
          }
        }
      ]
    },
    externals: {
      'fs': true,
      'path': true,
    }
  }