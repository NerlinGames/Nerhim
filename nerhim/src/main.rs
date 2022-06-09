mod game;

use winit::event::{DeviceEvent, WindowEvent};
use nokden::input::{InputSystem};
use enamorf::mesh::{MeshSystem};
use nokden::graphics::{GraphicsSystem};
use nokden::{ApplicationEvents, Framework, SystemEvents, SaveLoad};
use nokden::widgets::{ConsoleWidget, ConsoleState};
use game::{GameSystem};

fn main()
{
    nokden::run::<Application>();
}

pub struct Application
{
    game: game::GameSystem,
    input: InputSystem,
    meshes: MeshSystem,
    graphics: GraphicsSystem,

    //player: PlayerRig,

    console: ConsoleWidget   
}

impl ApplicationEvents
for Application
{
    fn name
    () 
    -> &'static str
    { 
        env!("CARGO_PKG_NAME")
    }

    fn version
    () 
    -> &'static str
    { 
        env!("CARGO_PKG_VERSION")
    }

    fn engine_name
    ()
    -> &'static str
    {
        enamorf::engine_name()
    }

    fn engine_version
    ()
    -> &'static str
    {
        enamorf::engine_version()
    }

    fn new
    (
        framework: &mut Framework
    )
    -> Application
    {
        let mut input = InputSystem::new();
        let mut graphics = GraphicsSystem::new(&framework.window());
        let mut meshes = MeshSystem::new(&graphics);

        let game = GameSystem::new(&mut input, &mut graphics, &mut meshes, framework);
        let console = ConsoleWidget::new(&mut input);                    

        Application
        {
            game,
            input,
            graphics,
            console,
            meshes
        }
    }

    fn update
    (
        &mut self,
        framework: &mut Framework        
    )
    {
        self.game.update(&mut self.input, &mut self.meshes, &mut self.graphics, framework);
        match self.console.update(&mut self.input, framework)
        {
            ConsoleState::Opened => (),
            ConsoleState::Closed => (),
        }        
    }

    fn update_engine
    (
        &mut self,
        device_event: &Vec<DeviceEvent>,
        window_event: &Vec<WindowEvent>
    )
    {
        for event in device_event
        {
            self.input.register_device_events(&event)
        }        

        let world_vp = self.graphics.world_camera.projection.as_matrix() * self.graphics.world_camera.transform.to_homogeneous();
        let frame_index = self.graphics.frame_start();
        self.meshes.update(&self.graphics, &world_vp);
        self.graphics.frame_end(frame_index);
    }    

    fn console // TODO Needs macro to run all.
    (
        &mut self,
        framework: &mut Framework
    )
    {
        self.game.console(framework);
        self.input.console(framework);
    }

    fn save_load // TODO Needs macro to run all.
    (
        &mut self,
        framework: &mut Framework
    )
    {
        match framework.save_load_event()
        {
            SaveLoad::Load => 
            {
                framework.save_load(SaveLoad::Idle);
                println!("Load");
            } 
            SaveLoad::Save => 
            {
                framework.save_load(SaveLoad::Idle);
                println!("Saved.")                
            }
            SaveLoad::Idle => ()
        }        
    }   
}