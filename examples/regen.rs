//! Regenerates the protobuf Rust code that is committed under `src/`.
//!
//! This example is the only thing in this crate that needs `protoc` on `PATH`. It is never run
//! during a normal build: `src/*.rs` already contains the generated modules, so consumers of this
//! crate can `cargo build` without `protoc` installed. Run `cargo run --example regen` from the
//! crate root after editing anything under `kytrix/`, then commit the updated `src/` files.

const PROTOS: &[&str] = &[
    "kytrix/client.proto",
    "kytrix/server.proto",
    "kytrix/event.proto",
    "kytrix/types/etf.proto",
    "kytrix/types/index.proto",
    "kytrix/types/x.proto",
    "kytrix/types/wallet.proto",
];

fn main() {
    let mut config = prost_build::Config::new();
    for ty in [
        ".kytrix.types.ETF",
        ".kytrix.types.LocalizedName",
        ".kytrix.types.IndexCategory",
        ".kytrix.types.IndexDefinition",
        ".kytrix.types.IndexManifest",
        ".kytrix.types.XProfile",
        ".kytrix.types.XSelection",
        ".kytrix.types.XSelectionStatus",
        ".kytrix.types.Tweet",
        ".kytrix.types.TweetTextSegment",
        ".kytrix.types.TweetMedia",
        ".kytrix.types.TweetBatch",
        ".kytrix.types.WalletNetwork",
        ".kytrix.types.WalletSelectionStatus",
        ".kytrix.types.WalletSelection",
        ".kytrix.types.WalletTransfer",
    ] {
        config.type_attribute(ty, "#[derive(serde::Serialize, serde::Deserialize)]");
    }
    config.out_dir("src");
    config
        .compile_protos(PROTOS, &["kytrix"])
        .expect("compile protos failed");
    println!("regenerated protos into src/");
}
