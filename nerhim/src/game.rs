use std::path::{PathBuf, Path};
use rayon::prelude::*;
use nalgebra::{Transform3, Matrix, Isometry3, Vector3};
use nokden::input::{InputSystem, Mapping, MethodKM};
use nokden::graphics::{GraphicsSystem};
use enamorf::{NodeSystem, Node};
use enamorf::mesh::{MeshSystem};
use nokden::{Handle, Framework, SystemEvents, SaveLoad, ConsoleCommandParameter, ConsoleCommand};

pub(crate) enum GameState
{
    InMenu,
    InGame,
    InConsole,
}

pub struct GameSystem
{
    pub(crate) state: GameState,

    pub(crate) rotate_mesh: Handle<Node>,
    pub(crate) bunch: Vec<Handle<Node>>,

    pub(crate) input_submit: Handle<Mapping>,
    pub(crate) input_close: Handle<Mapping>,    
    pub(crate) input_print_mapping: Handle<Mapping>,
    pub(crate) input_default_mapping: Handle<Mapping>,
    pub(crate) input_bind_mapping: Handle<Mapping>,
    pub(crate) input_load: Handle<Mapping>,
    pub(crate) input_save: Handle<Mapping>   
}

impl GameSystem
{
    pub(crate) fn new
    (        
        input: &mut InputSystem,
        nodes: &mut NodeSystem,
        graphics: &GraphicsSystem,
        meshes: &mut MeshSystem,
        framework: &mut Framework
    )    
    -> GameSystem
    {
        let input_submit = input.add_mapping(Mapping::new("GUI Submit", MethodKM::Enter));
        let input_close = input.add_mapping(Mapping::new("Quit App", MethodKM::ESC));        
        let input_print_mapping = input.add_mapping(Mapping::new("Print Mapping", MethodKM::F12));
        let input_default_mapping = input.add_mapping(Mapping::new("Default Mapping", MethodKM::F11));
        let input_bind_mapping = input.add_mapping(Mapping::new("Bind Mapping", MethodKM::F10));
        let input_load = input.add_mapping(Mapping::new("Load", MethodKM::F5));
        let input_save = input.add_mapping(Mapping::new("Save", MethodKM::F6));

        let rotate_mesh = nodes.add(Node::new());
        meshes.load_obj(framework.asset_path(Path::new("neticas.obj")), &graphics, nodes.storage.duplicate(&rotate_mesh));

        let bunch = 
        {
            let mut bunch: Vec<Handle<Node>> = Vec::new();
            for _ in 0..100000
            {
                let handle = nodes.add(Node::new());
                bunch.push(handle);
            }            
            bunch 
        };

        GameSystem
        {
            state: GameState::InMenu,
            bunch,
            input_submit,
            input_close,            
            input_print_mapping,
            input_default_mapping,
            input_bind_mapping,
            input_load,
            input_save,
            rotate_mesh
        }
    }

    pub(crate) fn update
    (
        &mut self,
        input: &mut InputSystem,
        framework: &mut Framework,
        nodes: &mut NodeSystem
    )
    {
        if input.check_once(&self.input_close)
        {
            framework.shutdown();
        }

        if input.check_once(&self.input_print_mapping)
        {
            input.print_mappings();
        }

        if input.check_once(&self.input_default_mapping)
        {
            input.default_mappings();
        }
        
        /*if self.input.check_once(&self.input_bind_mapping)
        {
            println!("Set custom bindings.");
            let mut mapping = self.input.mappings.write(&self.input_open_console);
            mapping.bind_custom(MethodKM::P);
        }*/

        if input.check_once(&self.input_load)
        {
            framework.save_load(SaveLoad::Load);
        }

        if input.check_once(&self.input_save)
        {
            framework.save_load(SaveLoad::Save);
        }

        self.bunch.par_iter_mut().for_each
        (
            |node|
            {
                let value = nodes.storage.read(node).matrix * 4.0;
            }
        );

        const ROTATE_SPEED_Y: f32 = 1.0;
        let mut node = nodes.storage.write(&self.rotate_mesh);
        let transform = Isometry3::<f32>::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, framework.delta() * ROTATE_SPEED_Y, 0.0));
        node.isometry = &node.isometry * transform;
    }
}

impl SystemEvents for GameSystem
{
    fn console
    (
        &mut self,
        framework: &mut Framework
    )
    {
        let test = ConsoleCommand::new("test", vec![ConsoleCommandParameter::Bool]);

        if framework.command_event() == &test
        {
            println!("Test command with parameter {}.", framework.command_event().get_bool(0).unwrap());
        }
    }

    fn save_load
    (
        &mut self,
        framework: &mut Framework
    )
    {
        println!("Saved.")    
    }
}