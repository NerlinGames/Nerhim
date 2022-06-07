use std::path::{PathBuf, Path};
use rayon::prelude::*;
use nalgebra::{Isometry3, Vector3, Point3, Matrix4, Translation3};
use nokden::input::{InputSystem, Mapping, MethodKM};
use nokden::graphics::{GraphicsSystem};
use enamorf::mesh::{MeshSystem, MeshInstance};
use nokden::*;

const MAP_SIZE: u8 = 25;
const TILE_METERS: f32 = 20.0;

pub(crate) enum GameState
{
    InMenu,
    InGame,
    InConsole,
}

pub struct GameSystem
{
    pub(crate) state: GameState,

    pub(crate) rotate_neticas: Handle<MeshInstance>,
    pub(crate) tiles: Vec<Handle<MeshInstance>>,

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
        graphics: &mut GraphicsSystem,
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

        graphics.world_projection.view = Isometry3::look_at_rh
        (
            &Point3::new(0.0, 30.0, -50.0),
            &Point3::origin(),
            &Vector3::y()
        );        

        let rotate_neticas =
        {
            let mesh_asset = meshes.load_asset_obj(framework.asset_path(Path::new("neticas.obj")), &graphics);
            meshes.instances.add
            (
                MeshInstance
                {
                    node: Isometry3::identity(),
                    mesh: mesh_asset
                }
            )
        };

        let tiles = 
        {
            let mesh_asset = meshes.load_asset_obj(framework.asset_path(Path::new("grass_tree.obj")), &graphics);
            let mut tiles: Vec<Handle<MeshInstance>> = Vec::new();
            let tile_count = MAP_SIZE as u64 * MAP_SIZE as u64;
            for index in 0..tile_count
            {
                let position = index.to_2D_square(MAP_SIZE as u64);
                let map_center = (MAP_SIZE / 2) as f32 * TILE_METERS + TILE_METERS / 2.0;
     
                tiles.push
                (
                    meshes.instances.add
                    (
                        MeshInstance
                        {
                            node: Isometry3::new
                            (
                                Vector3::new
                                (
                                    position[0] as f32 * TILE_METERS - map_center,
                                    0.0,
                                    position[1] as f32 * TILE_METERS - map_center,
                                ),
                                Vector3::zeros()
                            ),
                            mesh: meshes.assets.duplicate(&mesh_asset)
                        }
                    )
                );
            }            
            tiles 
        };

        GameSystem
        {
            state: GameState::InMenu,
            tiles,
            input_submit,
            input_close,            
            input_print_mapping,
            input_default_mapping,
            input_bind_mapping,
            input_load,
            input_save,
            rotate_neticas,
        }
    }

    pub(crate) fn update
    (
        &mut self,
        input: &mut InputSystem,
        meshes: &mut MeshSystem,
        framework: &mut Framework,
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

        if input.check_once(&self.input_load)
        {
            framework.save_load(SaveLoad::Load);
        }

        if input.check_once(&self.input_save)
        {
            framework.save_load(SaveLoad::Save);
        }

        // Multi-threading test.
        /*self.bunch.par_iter_mut().for_each
        (
            |node|
            {
                let value = nodes.storage.read(node).matrix * 4.0;
            }
        );*/       

        let mut instance = meshes.instances.write(&self.rotate_neticas);
        instance.node.delta_rotate
        (
            Point3::new(0.0, 30.0, 0.0),
            Vector3::new(0.0, 90.0, 0.0),
            framework.delta()
        );
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