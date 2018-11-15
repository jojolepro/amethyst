//! Different kinds of render passes.
//
pub use self::{
    debug_lines::*,
    flat::*,
    pbm::*,
    shaded::*,
    skinning::set_skinning_buffers,
    skybox::*,
    sprite::*,
    util::{get_camera, set_vertex_args},
};

mod debug_lines;
mod flat;
mod pbm;
mod shaded;
mod shaded_util;
mod skinning;
mod skybox;
mod sprite;
mod util;
