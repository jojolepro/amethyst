extern crate vergen;

fn main() {
    if let Err(err) = vergen::vergen(vergen::OutputFns::all()) {
        dontpanic!(
            "Vergen crate failed to generate version information! {:?}",
            err,
        );
    }
}
