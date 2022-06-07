pub mod mesh;

use nalgebra::{Matrix4, Isometry3};
use nokden::{Handle, Storage};

pub fn engine_name
()
-> &'static str
{
    env!("CARGO_PKG_NAME")
}

pub fn engine_version
()
-> &'static str
{
    env!("CARGO_PKG_VERSION")
}