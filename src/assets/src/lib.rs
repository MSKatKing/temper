mod resource;
mod holder;
mod location;

pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
