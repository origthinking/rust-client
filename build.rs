use std::io;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = "src/";
    tonic_prost_build::configure()
        .out_dir(out_dir)
        .compile_protos(
            &[
                "proto/qdrant.proto",
                "proto/collections.proto",
                "proto/collections_service.proto",
                "proto/points.proto",
                "proto/points_service.proto",
                "proto/snapshots_service.proto",
            ],
            &["proto"],
        )?;

    let qdrant_rs_path = Path::new(out_dir).join("qdrant.rs");
    let content = std::fs::read_to_string(&qdrant_rs_path)?;

    // This is the derive that prost-build adds by default for PointId.
    // We need to find it and remove `Hash`.
    let target_line = "#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]";
    let replacement_line = "#[derive(Clone, PartialEq, Eq, ::prost::Message)]";

    // To make this robust, we look for the derive attribute specifically on top of the PointId struct definition.
    let target_block = format!("{}\npub struct PointId", target_line);
    let replacement_block = format!("{}\npub struct PointId", replacement_line.replace(", Hash", ""));

    let new_content = content.replace(&target_block, &replacement_block);

    if new_content == content {
        // If the replacement did not happen, it means the generated code has changed.
        // We should fail the build loudly.
        // A more robust solution might be to parse the file, but this is a good first step.
        let updated_target_line = target_line.replace(", Hash", "");
        let updated_target_block = format!("{}\npub struct PointId", updated_target_line);

        if !content.contains(&updated_target_block) {
             return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "build.rs failed to remove the Hash derive from the PointId struct. The prost-build output may have changed.",
            )));
        }
    }

    std::fs::write(&qdrant_rs_path, new_content)?;

    Ok(())
}