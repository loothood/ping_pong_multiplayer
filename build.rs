fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["src/proto/game.proto"],
            &["src/proto"],
        ).unwrap();
    Ok(())
}