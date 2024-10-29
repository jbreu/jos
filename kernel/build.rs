extern crate vergen;

fn main() {
    let build = vergen::BuildBuilder::all_build().expect("failed to build vergen");

    vergen::Emitter::default()
        .add_instructions(&build)
        .unwrap()
        .emit()
        .unwrap();
}
