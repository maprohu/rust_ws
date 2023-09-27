fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/hub.proto")?;
    tonic_build::compile_protos("proto/log.proto")?;
    Ok(())
}