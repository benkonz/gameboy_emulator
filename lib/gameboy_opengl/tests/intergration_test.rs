extern crate gameboy_opengl;

#[cfg(test)]
mod tests {
    use gameboy_opengl;

    #[test]
    fn test_tetris() {
        let bytes = include_bytes!("drmario.gb");

        gameboy_opengl::start(bytes.to_vec());
    }
}