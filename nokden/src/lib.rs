pub mod defaults;
pub mod input;
pub mod widgets;
pub mod graphics;

use std::{env, fs};
use std::marker::PhantomData;
use std::path::{PathBuf, Path};
use std::time::Instant;
use std::sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard};
use winit::window::{Window, WindowBuilder};
use winit::event::{WindowEvent, DeviceEvent};
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run
<
    TApplication: ApplicationEvents + 'static
>
()
{
    let event_loop = EventLoop::new();
    let console_quit = ConsoleCommand::new("quit", Vec::new());
    let console_save = ConsoleCommand::new("save", Vec::new());
    let console_load = ConsoleCommand::new("load", Vec::new());
    let mut device_events: Vec<DeviceEvent> = Vec::new();
    let mut window_events: Vec<WindowEvent> = Vec::new();    

    println!("Application:\t{} ({})", TApplication::name(), TApplication::version());
    println!("Engine:\t\t{} ({})", TApplication::engine_name(), TApplication::engine_version());
    println!("Framework:\t{} ({})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let mut framework = Framework::new(&event_loop, TApplication::name());
    let mut application = TApplication::new(&mut framework);
    
    println!();
    println!("Enter loop ...");

    framework.window_show();

    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
            winit::event::Event::MainEventsCleared =>
            {
                match *control_flow
                {
                    ControlFlow::Poll =>
                    {
                        if !framework.run
                        {
                            *control_flow = ControlFlow::Exit
                        }

                        if !framework.command.keyword.is_empty()
                        {  
                            if framework.command == console_quit
                            {
                                framework.shutdown();
                            }

                            if framework.command == console_save
                            {
                                framework.save_load(SaveLoad::Save)
                            }

                            if framework.command == console_load
                            {
                                framework.save_load(SaveLoad::Load)
                            }

                            application.console(&mut framework);
                            framework.command.keyword.clear();
                        }
                        
                        application.update(&mut framework);
                        application.update_engine(&device_events, &window_events);
                        application.save_load(&mut framework);
                        device_events.clear();
                        window_events.clear();
                        framework.frame_delta.reset();
                        framework.frames.count();
                        framework.fps.count(false);
                    }
                    ControlFlow::Exit => 
                    {
                        //println!("... app shutdown update ...");
                    },
                    _ => unreachable!()
                }               
            }
            winit::event::Event::DeviceEvent { event, .. } =>
            {
                device_events.push(event)                
            }
            winit::event::Event::WindowEvent { event, .. } =>
            {
                match event
                {
                    WindowEvent::CloseRequested =>
                    {
                        *control_flow = ControlFlow::Exit;                    
                    }                    
                    WindowEvent::Resized(size) =>
                    {
                        //application.write().unwrap().graphics_core().window_changed(size.width, size.height);
                    }
                    WindowEvent::CursorMoved { position, .. } =>
                    {
                        //let mut inst = app_instance.write().unwrap();
                        //let window_size = inst.core.window_size();
                        //inst.input.register_signal_mouse_cursor([position.x, position.y], window_size);
                    }
                    _ => ()
                }
            }
            winit::event::Event::LoopDestroyed =>
            {
                println!();
                println!("... loop done. Engine shut down.");
            }
            _ => ()
        }
    });
}

pub struct Storage
<
    TComponent
>
{
    regsiter: Vec<Register>,
    usage_count: Vec<u32>,
    datas: Vec<Arc<RwLock<TComponent>>>
}

impl
<
    TComponent
>
Storage
<
    TComponent
>
{
    pub fn new
    ()
    -> Storage<TComponent>
    {
        Storage
        {
            regsiter: Vec::new(),
            datas: Vec::new(),
            usage_count: Vec::new()
        }
    }

    pub fn add
    (
        &mut self,
        data: TComponent
    )   
    -> Handle<TComponent> 
    {
        self.datas.push(Arc::new(RwLock::new(data)));
        self.usage_count.push(1);

        Handle
        {
            index: (self.datas.len()) - 1,
            phantom: PhantomData::default()
        }
    }

    pub fn duplicate
    (
        &mut self,
        handle: &Handle<TComponent>        
    )
    -> Handle<TComponent>        
    {
        self.usage_count[handle.index] += 1;
        Handle         
        { 
            index: handle.index,
            phantom: PhantomData::default()
        }
    }

    pub fn remove
    (
        &mut self,
        handle: Handle<TComponent>
    )
    {
        todo!()//self.datas.remove(handle.register_index);
    }

    pub fn read
    (
        &self,
        handle: &Handle<TComponent>
    )
    -> RwLockReadGuard<TComponent>
    {
        self.datas.get(handle.index).unwrap().read().unwrap()
    }

    pub fn write
    (
        &mut self,
        handle: &Handle<TComponent>
    )
    -> RwLockWriteGuard<TComponent>
    {
        self.datas.get(handle.index).unwrap().write().unwrap()
    }

    pub fn all
    (
        &self
    )
    -> &[Arc<RwLock<TComponent>>]
    {
        self.datas.as_slice()
    }
}

struct RegisterInfo
{
    data_index: u8,
    
    /// Counted when added and removed.
    generation: u8
}

enum Register
{
    Disabled(RegisterInfo),
    Enabled(RegisterInfo),
    //Streaming,
    Removed(RegisterInfo)
}

pub struct Handle
<
    TComponent
>
{
    index: usize,
    phantom: PhantomData<TComponent>
}

pub enum SaveLoad
{
    Idle,
    Load,
    Save
}

pub struct Framework
{
    asset_path: PathBuf,
    fps: CPS,
    frames: Frames,
    frame_delta: Delta,
    window: Window, // TODO Needs to be moved to graphics core.
    run: bool,
    command: ConsoleCommand,
    save_load: SaveLoad
}

impl Framework
{
    pub fn new
    (
        event_loop: &EventLoop<()>,
        app_name: &str
    )
    -> Framework
    {
        let window =
        {
            // TODO Needs to be the OS set desktop resolution.
            let (width, height) = (defaults::RESOLUTION_WIDTH, defaults::RESOLUTION_HEIGHT);
            let window = WindowBuilder::new().build(&event_loop).unwrap();
            window.set_visible(false);
            window.set_title(app_name);
            window.set_inner_size(winit::dpi::LogicalSize::new(f64::from(width), f64::from(height)));
            window
        };

        let asset_path =
        {
            const DROP_FOLDERS: usize = 4;

            let from_source = match env::current_exe().unwrap().iter().find(|item| *item == "target")
            {
                Some(_) => true,
                None => false
            };

            let asset_path =
            {
                let mut source_directory_exe = env::current_exe().unwrap();
                match from_source
                {
                    true => for _ in 0..DROP_FOLDERS { source_directory_exe.pop(); },
                    false => { source_directory_exe.pop(); }
                }
                source_directory_exe.push("@main");
                source_directory_exe.push("import");
                source_directory_exe
            };

            if from_source
            {
                let source_directory_exe = env::current_exe().unwrap();

                let mut project_directory_exe = env::current_exe().unwrap();                
                for _ in 0..DROP_FOLDERS { project_directory_exe.pop(); }
                project_directory_exe.push(&env::current_exe().unwrap().components().last().unwrap().as_os_str().to_str().unwrap());
                project_directory_exe.pop();
                project_directory_exe.push(Path::new(format!("launch {}", app_name).as_str()));

                match fs::copy(&source_directory_exe, &project_directory_exe)
                {
                    Ok(_) => println!("From source: {}", from_source),
                    Err(err) =>
                    {
                        println!("App exe not copied: {}", err);
                        println!("From: {}", &source_directory_exe.to_string_lossy());
                        println!("To: {}", &project_directory_exe.to_string_lossy());
                    }
                }
            }

            asset_path
        };

        Framework
        {
            asset_path,
            fps: CPS::new("Frames Per Second"),
            frames: Frames::new(),
            frame_delta: Delta::new(),
            window,
            run: true,
            command: ConsoleCommand::cleared(),
            save_load: SaveLoad::Idle
        }
    }
    
    pub fn shutdown
    (
        &mut self,
    )
    {
        self.run = false;
    }

    /// Issue console command.
    pub fn command
    (
        &mut self,
        command: &str
    )
    {
        let command_string = String::from(command);
        let mut fragments: Vec<&str> = command_string.split(' ').collect();
        let mut parameters: Vec<ConsoleCommandParameter> = Vec::new();

        let keyword = fragments.remove(0);

        for fragment in fragments
        {
            // TODO Once the if let is supported inside match by Rust then refactor instead of using else-if chains.
            /*if let Some(value) = fragment.parse::<bool>().ok()
            {
                parameters.push(ConsoleCommandParameter::Bool(value));
            }
            else if let Some(value) = fragment.parse::<u32>().ok()
            {
                parameters.push(ConsoleCommandParameter::U32(value));
            }
            else if let Some(value) = fragment.parse::<f32>().ok()
            {
                parameters.push(ConsoleCommandParameter::F32(value));
            }*/
        }        

        self.command = ConsoleCommand::new(keyword, parameters);
    }

    /// Get issued command event.
    pub fn command_event
    (
        &self,
    )
    -> &ConsoleCommand
    {
        &self.command
    }

    pub fn save_load
    (
        &mut self,
        mode: SaveLoad
    )
    {
        self.save_load = mode;
    }

    /// Get issued save or load event.
    pub fn save_load_event
    (
        &self
    )
    -> &SaveLoad
    {
        &self.save_load
    }

    pub fn window_show
    (
        &mut self
    )
    {
        self.window.set_maximized(false);
        self.window.set_visible(true);
    }

    pub fn window
    (
        &self
    )
    -> &Window
    {
        &self.window
    }

    pub fn window_size
    (
        &self
    )
    -> [u32; 2]
    {
        [self.window.inner_size().width, self.window.inner_size().height]
    }

    pub fn delta
    (
        &self
    )
    -> f32
    {
        self.frame_delta.delta()
    }

    pub fn asset_path
    (
        &self,
        asset: &Path
    )
    -> AssetPath
    {
        let mut full_path = self.asset_path.clone();
        full_path.push(asset);
        AssetPath(full_path.clone())
    }
}

pub enum ConsoleCommandParameter
{
    //Keyword(String),
    Bool,
    U32,
    F32
}

pub struct ConsoleCommand
{
    keyword: String,
    parameters: Vec<ConsoleCommandParameter>
}

impl ConsoleCommand
{
    pub fn new
    (
        keyword: &str,
        parameters: Vec<ConsoleCommandParameter>
    )    
    -> ConsoleCommand
    {
        ConsoleCommand
        {
            keyword: String::from(keyword),
            parameters
        }
    }    

    fn cleared
    ()
    -> ConsoleCommand
    {
        ConsoleCommand
        {
            keyword: String::new(),
            parameters: Vec::new()
        }
    }

    /// Get a parameter.
    /// Returns none if there is no paramater or of the wrong type.
    // TODO Perhaps getting can be replaced with one single generic trait based method that checks at compile time to.
    pub fn get_bool
    (
        &self,
        index: usize
    )
    -> Option<bool>
    {
        /*if let ConsoleCommandParameter::Bool(value) = self.parameters.get(index).unwrap()
        {
            Some(*value)
        }
        else
        {
            None
        }*/
        unimplemented!()
    }
}

impl PartialEq for ConsoleCommand
{
    fn eq
    (
        &self,
        other: &Self
    )
    -> bool
    {
        const INFO_EVENT: &str = "info";

        if self.keyword == INFO_EVENT || other.keyword == INFO_EVENT
        {
            println!("\t{}", other.keyword);
            false
        }
        else
        {
            self.keyword == other.keyword
        }
    }
}

pub struct AssetPath(pub PathBuf);

pub trait ApplicationEvents
{
    fn name
    ()
    -> &'static str;

    fn version
    ()
    -> &'static str;
    
    fn engine_name
    ()
    -> &'static str;

    fn engine_version
    ()
    -> &'static str;

    fn new
    (
        framework: &mut Framework
    )
    -> Self;
    
    fn update
    (
        &mut self,
        framework: &mut Framework
    );
    
    fn update_engine
    (
        &mut self,
        device_event: &Vec<DeviceEvent>,
        window_event: &Vec<WindowEvent>
    );

    fn console
    (
        &mut self,
        framework: &mut Framework
    );

    fn save_load
    (
        &mut self,
        framework: &mut Framework
    );
}

pub trait SystemEvents
{
    fn console
    (
        &mut self,
        framework: &mut Framework
    );

    fn save_load
    (
        &mut self,
        framework: &mut Framework
    );
}

/// Counts Per Second.
/// For instance this can be used to count frames per second.
pub struct CPS
{
    label: String,
    current: u32,
    per_second: Instant,
    counts: u32
}

impl CPS
{
    pub fn new
    (
        name: &str
    )
    -> CPS
    {
        CPS
        {
            label: name.to_string(),
            current: 0,
            per_second: Instant::now(),
            counts: 0
        }
    }

    pub fn count
    (
        &mut self,
        print: bool
    )
    {
        self.refresh_per_second(print);
        self.counts += 1;
    }

    pub fn current
    (
        &mut self
    )
    -> u32
    {
        self.refresh_per_second(false);
        self.current
    }

    /// Print every second to console if [print] is true.
    fn refresh_per_second
    (
        &mut self,
        print: bool // TODO Needs to be enabled via internal bool and pub function instead.
    )
    {
        if self.per_second.elapsed().as_secs_f32() >= 1.0
        {
            self.current = self.counts;
            self.counts = 0;
            self.per_second = Instant::now();

            if print
            {
                println!("{}: {}", self.label, self.current)
            }
        }
    }
}

pub struct Frames(u32);

impl Frames
{
    pub fn new
    ()
    -> Frames
    {
        Frames(0)
    }

    pub fn count
    (
        &mut self
    )
    {
        self.0 += 1;
    }

    pub fn clear
    (
        &mut self
    )
    {
        self.0 = 0;
    }

    pub fn frames(&self) -> u32
    {
        self.0
    }
}

pub struct Delta(Instant);

impl Delta
{
    pub fn new
    ()
    -> Delta
    {
        Delta(Instant::now())
    }

    pub fn reset
    (
        &mut self
    )
    {
        self.0 = Instant::now()
    }

    pub fn delta
    (
        &self
    ) -> f32
    {
        self.0.elapsed().as_secs_f32()
    }
}

#[cfg(debug_assertions)]
pub fn platform
()
-> String
{
    "Linux x86_64 Debug".to_string()
}

#[cfg(not(debug_assertions))]
fn platform
()
-> String
{
    "Linux x86_64".to_string()
}

/// Simple offset_of macro akin to C++ offsetof.
#[macro_export]
macro_rules! offset_of
{
    (
        $base:path,
        $field:ident
    )
    =>
    {
        {
            #[allow(unused_unsafe)]
            unsafe
            {
                let b: $base = mem::zeroed();
                (&b.$field as *const _ as isize) - (&b as *const _ as isize)
            }
        }
    };
}

/// Generate a name()-> 'static str function for every enum variant.
#[macro_export]
macro_rules! enum_str // TODO Needs to be more fexliable regarding pub, derive, and variant values.
{
    (
        #[derive(Copy, Clone)]
        pub enum $name:ident
        {
            $($variant:ident),*,
        }
    ) =>
    {
        #[derive(Copy, Clone)]
        pub enum $name 
        {
            $($variant),*
        }

        impl $name 
        {
            fn make_str
            (
                &self
            )
            -> &'static str 
            {
                match self
                {
                    $($name::$variant => stringify!($variant)),*
                }
            }
        }
    };
}

/// Find all fields with a specific trait. // TODO What are the specific traits?
#[macro_export]
macro_rules! find_traits
{
    (
        $field:ident
    )
    =>
    {

    };
}