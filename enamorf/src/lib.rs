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

pub struct NodeSystem
{
    pub storage: Storage<Node>
}

impl NodeSystem
{
    pub fn new
    ()
    -> NodeSystem
    {
        NodeSystem
        {
            storage: Storage::new()
        }
    }

    pub fn add
    (
        &mut self,
        node: Node
    )
    -> Handle<Node>
    {
        let node = self.storage.add(node);
        //println!("Add: {:?}", node);
        node
    }
}

#[derive(Debug)]
pub struct Node
{
    pub enable: bool,
    pub matrix: Matrix4<f32>,
    pub isometry: Isometry3<f32>,
    pub subtus: Vec<Handle<Node>>
}

impl Node
{
    pub fn new
    ()
    -> Node
    {
        Node
        {
            enable: true,
            matrix: Matrix4::identity(),
            isometry: Isometry3::identity(),
            subtus: Vec::new()
        }
    }

    pub fn attach
    (
        &mut self,
        subtus: Handle<Node>
    )
    {
        self.subtus.push(subtus)
    }
}