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
    root: Handle<Node>,
    pub storage: Storage<Node>
}

impl NodeSystem
{
    pub fn new
    (
        root_node: Node
    )
    -> NodeSystem
    {
        let mut nodes = Storage::new();
        let root = nodes.add(root_node);

        NodeSystem
        {
            root,
            storage: nodes
        }
    }

    pub fn add
    (
        &mut self,
        node: Node
    )
    -> Handle<Node>
    {
        self.storage.add(node)
    }

    pub fn attach_to_root
    (
        &mut self,
        subtus: Handle<Node>
    )
    {
        let mut root_node = self.storage.write(&self.root);
        root_node.attach(subtus);
        //self.root.access_mut().attach(&subtus)
    }
}

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