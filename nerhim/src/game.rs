use std::path::{PathBuf, Path};
use rayon::prelude::*;
use nalgebra::{Isometry3, Vector3, Point3, Matrix4, Translation3};
use nokden::input::{InputSystem, Mapping, MethodKM};
use nokden::graphics::{GraphicsSystem};
use enamorf::mesh::{MeshSystem, MeshInstance};
use nokden::*;

const CAMERA_SPEED: f32 = 10.0;

const MAP_SIZE: u8 = 30;
const TILE_METERS: f32 = 1.0;

pub(crate) enum GameState
{
    InMenu,
    InGame,
    InConsole,
}

pub struct GameSystem
{
    state: GameState,

    rotate_neticas: Handle<MeshInstance>,
    tiles: Vec<Handle<MeshInstance>>,

    input_camera_forward: Handle<Mapping>,
    input_camera_backward: Handle<Mapping>,
    input_camera_right: Handle<Mapping>,    
    input_camera_left: Handle<Mapping>,

    input_submit: Handle<Mapping>,
    input_close: Handle<Mapping>,    
    input_print_mapping: Handle<Mapping>,
    input_default_mapping: Handle<Mapping>,
    input_bind_mapping: Handle<Mapping>,
    input_load: Handle<Mapping>,
    input_save: Handle<Mapping>   
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
        graphics.world_camera.transform = Isometry3::look_at_rh
        (
            &Point3::new(0.0, 15.0, -10.0),
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
                    transform: Isometry3::identity(),
                    mesh: mesh_asset
                }
            )
        };

        let tiles = 
        {
            let mesh_asset = meshes.load_asset_obj(framework.asset_path(Path::new("tile_test.obj")), &graphics);
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
                            transform: Isometry3::new
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
            state: GameState::InGame,
            tiles,
            rotate_neticas,
            input_submit: input.add_mapping(Mapping::new("GUI Submit", MethodKM::Enter)),
            input_close: input.add_mapping(Mapping::new("Quit App", MethodKM::ESC)),            
            input_print_mapping: input.add_mapping(Mapping::new("Print Mapping", MethodKM::F12)),
            input_default_mapping: input.add_mapping(Mapping::new("Default Mapping", MethodKM::F11)),
            input_bind_mapping: input.add_mapping(Mapping::new("Bind Mapping", MethodKM::F10)),
            input_load: input.add_mapping(Mapping::new("Load", MethodKM::F5)),
            input_save: input.add_mapping(Mapping::new("Save", MethodKM::F6)),            
            input_camera_forward: input.add_mapping(Mapping::new("Move Camera Forward", MethodKM::W)),
            input_camera_backward: input.add_mapping(Mapping::new("Move Camera Backward", MethodKM::S)),
            input_camera_right: input.add_mapping(Mapping::new("Move Camera Right", MethodKM::D)),
            input_camera_left: input.add_mapping(Mapping::new("Move Camera Left", MethodKM::A)),
        }
    }

    pub(crate) fn update
    (
        &mut self,
        input: &mut InputSystem,
        meshes: &mut MeshSystem,
        graphics: &mut GraphicsSystem,
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

        match self.state
        {
            GameState::InGame =>
            {
                /*if input.check_once(&self.input_camera_forward)
                {
                    graphics.world_camera.transform.append_translation_mut(&Translation3::new(0.0, 0.0, 10.0));
                }

                if input.check_once(&self.input_camera_backward)
                {
                    
                }

                if input.check_once(&self.input_camera_forward)
                {
                    
                }

                if input.check_once(&self.input_camera_forward)
                {
                    
                }*/
            }
            _ => todo!()
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
        instance.transform.delta_rotate
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