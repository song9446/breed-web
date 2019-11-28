//extern crate flatc_rust;
use std::path::Path;

fn main() {
    prost_build::compile_protos(&["src/proto/event.proto", "src/proto/action.proto"],
                                &["src/"]).unwrap();
}
