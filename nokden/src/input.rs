use winit::event::{DeviceEvent, VirtualKeyCode, ElementState};
use crate::{CPS, Handle, Storage, enum_str, SystemEvents, ConsoleCommand, Framework, ConsoleCommandParameter};

pub struct InputSystem
{
    mode: Mode,
    signals_km: [Signal; MAX_SIGNAL_SLOTS_KM],
    //signals_gp: [Signal; SIGNAL_SLOTS],
    signals_mouse_cursor: [f32; 2],    

    /// Signals Per Second.
    pub sps: CPS,

    pub mappings: Storage<Mapping>
}

impl InputSystem
{
    pub fn new
    ()
    -> InputSystem
    {
        let mut mappings = Storage::new();
        let submit = mappings.add(Mapping::new("(GUI) Submit", MethodKM::Enter));
        let typing_cancel = mappings.add(Mapping::new("(GUI) Cancel Typing", MethodKM::ESC));

        InputSystem
        {
            mode: Mode::Normal,
            signals_km: [Signal::Inactive; MAX_SIGNAL_SLOTS_KM],
            signals_mouse_cursor: [0.0, 0.0],
            sps: CPS::new("Input Signals Per Second"),
            mappings
        }
    }

    pub fn mode
    (
        &mut self,
        mode: Mode
    )
    {
        self.mode = mode;
    }

    pub fn add_mapping // TODO Needs to be removed.
    (
        &mut self,
        mapping: Mapping
    )
    -> Handle<Mapping>
    {
        self.mappings.add(mapping)
    }

    pub fn check
    (
        &mut self,
        mapping: &Handle<Mapping>
    )
    -> bool
    {
        match self.mode
        {
            Mode::Normal =>
            {
                /*let mut once = false;
                
                let mapping = self.mappings.read(mapping);
                let slot = match mapping.custom
                {
                    Some(custom_mapping) => *(&custom_mapping) as usize,
                    None => *(&mapping.default) as usize
                };

                once = match self.signals_km[slot]
                {
                    Signal::Active(_, checked_once) =>
                    {
                        match checked_once
                        {
                            CheckedOnce(false) =>
                            {
                                self.signals_km[slot] = Signal::Active(0.0, CheckedOnce(true));
                                true
                            }
                            CheckedOnce(true) => false
                        }
                    }
                    Signal::Inactive =>
                    {
                        false
                    }
                };

                once*/
                self.check_once(mapping)
            }
            _ => false
        }
    }

    pub fn check_hotkey
    (
        &mut self,
        mapping: &Handle<Mapping>
    )
    -> bool
    {
        match self.mode
        {
            Mode::HotKey =>
            {
                self.check_once(mapping)
            }
            _ => false
        }
    }

    /*fn check_once
    (
        signals_km: &[Signal],
        mapping: &Handle<Mapping>
    )
    -> bool
    {
        let mut once = false;
                
        let mapping = self.mappings.read(mapping);
        let slot = match mapping.custom
        {
            Some(custom_mapping) => *(&custom_mapping) as usize,
            None => *(&mapping.default) as usize
        };

        once = match self.signals_km[slot]
        {
            Signal::Active(_, checked_once) =>
            {
                match checked_once
                {
                    CheckedOnce(false) =>
                    {
                        self.signals_km[slot] = Signal::Active(0.0, CheckedOnce(true));
                        true
                    }
                    CheckedOnce(true) => false
                }
            }
            Signal::Inactive =>
            {
                false
            }
        };

        once
    }*/

    pub fn check_once // TODO Needs to be combined with [check] function.
    (
        &mut self,
        mapping: &Handle<Mapping>
    )
    -> bool
    {        
        let mapping = self.mappings.read(mapping);
        match mapping.custom
        {
            Some(custom_mapping) => Self::check_once_signal(&mut self.signals_km[custom_mapping as usize]),
            None => Self::check_once_signal(&mut self.signals_km[mapping.default as usize])
        }
    }

    /*pub fn check_constant
    (
        &mut self,
        binding: &resources::Accessor<Binding>
    )
    -> Option<f32>
    {
        match self.mode
        {
            Mode::ConstantAndOnce =>
                {
                    let mut constant = None;

                    let binding = self.bindings.access(binding);
                    let default = &binding.default;
                    let slot = *default as usize;

                    constant = match self.signals_km[slot]
                    {
                        Signal::Active(data, _) => Some(data),
                        Signal::Inactive => None
                    };

                    constant
                }
            _ => None
        }
    }*/

    pub fn check_cursor_position
    (
        &self
    )
    -> [f32; 2]
    {
        self.signals_mouse_cursor
    }

    pub fn check_typing
    (
        &mut self,
        typing: &mut String
    )
    {
        match self.mode
        {            
            Mode::TypingNLS => 
            {                
                for (slot, signal) in self.signals_km.iter_mut().enumerate()
                {
                    if slot == MethodKM::F1 as usize {()}
                    else if slot == MethodKM::F2 as usize {()}
                    else if slot == MethodKM::Enter as usize {()}
                    else if slot == MethodKM::TAB as usize {()}
                    else if slot == MethodKM::Backspace as usize && Self::check_once_signal(signal)
                    {
                        typing.pop();
                    }
                    else if Self::check_once_signal(signal)
                    {
                        if slot == MethodKM::A as usize { *typing += "a" }
                        else if slot == MethodKM::B as usize { *typing += "b" }
                        else if slot == MethodKM::C as usize { *typing += "c" }
                        else if slot == MethodKM::D as usize { *typing += "d" }
                        else if slot == MethodKM::E as usize { *typing += "e" }
                        else if slot == MethodKM::F as usize { *typing += "f" }
                        else if slot == MethodKM::G as usize { *typing += "g" }
                        else if slot == MethodKM::H as usize { *typing += "h" }
                        else if slot == MethodKM::I as usize { *typing += "i" }
                        else if slot == MethodKM::J as usize { *typing += "j" }
                        else if slot == MethodKM::K as usize { *typing += "k" }
                        else if slot == MethodKM::L as usize { *typing += "l" }
                        else if slot == MethodKM::M as usize { *typing += "m" }
                        else if slot == MethodKM::N as usize { *typing += "n" }
                        else if slot == MethodKM::O as usize { *typing += "o" }
                        else if slot == MethodKM::P as usize { *typing += "p" }
                        else if slot == MethodKM::Q as usize { *typing += "q" }
                        else if slot == MethodKM::R as usize { *typing += "r" }
                        else if slot == MethodKM::S as usize { *typing += "s" }
                        else if slot == MethodKM::T as usize { *typing += "t" }
                        else if slot == MethodKM::U as usize { *typing += "u" }
                        else if slot == MethodKM::V as usize { *typing += "v" }
                        else if slot == MethodKM::W as usize { *typing += "w" }
                        else if slot == MethodKM::X as usize { *typing += "x" }
                        else if slot == MethodKM::Y as usize { *typing += "y" }
                        else if slot == MethodKM::Z as usize { *typing += "z" }
                        else if slot == MethodKM::Key0 as usize { *typing += "0" }
                        else if slot == MethodKM::Key1 as usize { *typing += "1" }
                        else if slot == MethodKM::Key2 as usize { *typing += "2" }
                        else if slot == MethodKM::Key3 as usize { *typing += "3" }
                        else if slot == MethodKM::Key4 as usize { *typing += "4" }
                        else if slot == MethodKM::Key5 as usize { *typing += "5" }
                        else if slot == MethodKM::Key6 as usize { *typing += "6" }
                        else if slot == MethodKM::Key7 as usize { *typing += "7" }
                        else if slot == MethodKM::Key8 as usize { *typing += "8" }
                        else if slot == MethodKM::Key9 as usize { *typing += "9" }
                        else if slot == MethodKM::Space as usize { *typing += " " }
                    }                   
                }
            }
            _ => (),
        }       
    }

    fn check_once_signal
    (
        signal: &mut Signal
    )
    -> bool
    {
        match signal
        {
            Signal::Inactive => false,
            Signal::Active(_, checked_once) => 
            {                            
                match checked_once
                {
                    CheckedOnce(false) =>
                    {
                        *signal = Signal::Active(0.0, CheckedOnce(true));                        
                        true
                    }
                    CheckedOnce(true) => false
                }
            }
        } 
    }

    pub fn register_device_events
    (
        &mut self,
        input: &DeviceEvent
    )
    {
        match input // TODO Seems that the polling rate is 30hz only. That's a big problem for 60+ FPS games.
        {
            DeviceEvent::Key (keyboard) =>
            {
                match keyboard.virtual_keycode
                {
                    Some(active) =>
                    {
                        match active
                        {
                            VirtualKeyCode::Escape => self.register_signal_km(MethodKM::ESC as usize, &keyboard.state),
                            VirtualKeyCode::Tab => self.register_signal_km(MethodKM::TAB as usize, &keyboard.state),
                            VirtualKeyCode::Space => self.register_signal_km(MethodKM::Space as usize, &keyboard.state),
                            VirtualKeyCode::Back => self.register_signal_km(MethodKM::Backspace as usize, &keyboard.state),
                            VirtualKeyCode::Return => self.register_signal_km(MethodKM::Enter as usize, &keyboard.state),
                            VirtualKeyCode::LShift => self.register_signal_km(MethodKM::ShiftLeft as usize, &keyboard.state),
                            VirtualKeyCode::RShift => self.register_signal_km(MethodKM::ShiftRight as usize, &keyboard.state),

                            VirtualKeyCode::F1 => self.register_signal_km(MethodKM::F1 as usize, &keyboard.state),
                            VirtualKeyCode::F2 => self.register_signal_km(MethodKM::F2 as usize, &keyboard.state),
                            VirtualKeyCode::F3 => self.register_signal_km(MethodKM::F3 as usize, &keyboard.state),
                            VirtualKeyCode::F4 => self.register_signal_km(MethodKM::F4 as usize, &keyboard.state),
                            VirtualKeyCode::F5 => self.register_signal_km(MethodKM::F5 as usize, &keyboard.state),
                            VirtualKeyCode::F6 => self.register_signal_km(MethodKM::F6 as usize, &keyboard.state),
                            VirtualKeyCode::F7 => self.register_signal_km(MethodKM::F7 as usize, &keyboard.state),
                            VirtualKeyCode::F8 => self.register_signal_km(MethodKM::F8 as usize, &keyboard.state),
                            VirtualKeyCode::F9 => self.register_signal_km(MethodKM::F9 as usize, &keyboard.state),
                            VirtualKeyCode::F10 => self.register_signal_km(MethodKM::F10 as usize, &keyboard.state),
                            VirtualKeyCode::F11 => self.register_signal_km(MethodKM::F11 as usize, &keyboard.state),
                            VirtualKeyCode::F12 => self.register_signal_km(MethodKM::F12 as usize, &keyboard.state),

                            VirtualKeyCode::Key0 => self.register_signal_km(MethodKM::Key0 as usize, &keyboard.state),
                            VirtualKeyCode::Key1 => self.register_signal_km(MethodKM::Key1 as usize, &keyboard.state),
                            VirtualKeyCode::Key2 => self.register_signal_km(MethodKM::Key2 as usize, &keyboard.state),
                            VirtualKeyCode::Key3 => self.register_signal_km(MethodKM::Key3 as usize, &keyboard.state),
                            VirtualKeyCode::Key4 => self.register_signal_km(MethodKM::Key4 as usize, &keyboard.state),
                            VirtualKeyCode::Key5 => self.register_signal_km(MethodKM::Key5 as usize, &keyboard.state),
                            VirtualKeyCode::Key6 => self.register_signal_km(MethodKM::Key6 as usize, &keyboard.state),
                            VirtualKeyCode::Key7 => self.register_signal_km(MethodKM::Key7 as usize, &keyboard.state),
                            VirtualKeyCode::Key8 => self.register_signal_km(MethodKM::Key8 as usize, &keyboard.state),
                            VirtualKeyCode::Key9 => self.register_signal_km(MethodKM::Key9 as usize, &keyboard.state),

                            VirtualKeyCode::A => self.register_signal_km(MethodKM::A as usize, &keyboard.state),
                            VirtualKeyCode::B => self.register_signal_km(MethodKM::B as usize, &keyboard.state),
                            VirtualKeyCode::C => self.register_signal_km(MethodKM::C as usize, &keyboard.state),
                            VirtualKeyCode::D => self.register_signal_km(MethodKM::D as usize, &keyboard.state),
                            VirtualKeyCode::E => self.register_signal_km(MethodKM::E as usize, &keyboard.state),
                            VirtualKeyCode::F => self.register_signal_km(MethodKM::F as usize, &keyboard.state),
                            VirtualKeyCode::G => self.register_signal_km(MethodKM::G as usize, &keyboard.state),
                            VirtualKeyCode::H => self.register_signal_km(MethodKM::H as usize, &keyboard.state),
                            VirtualKeyCode::I => self.register_signal_km(MethodKM::I as usize, &keyboard.state),
                            VirtualKeyCode::J => self.register_signal_km(MethodKM::J as usize, &keyboard.state),
                            VirtualKeyCode::K => self.register_signal_km(MethodKM::K as usize, &keyboard.state),
                            VirtualKeyCode::L => self.register_signal_km(MethodKM::L as usize, &keyboard.state),
                            VirtualKeyCode::M => self.register_signal_km(MethodKM::M as usize, &keyboard.state),
                            VirtualKeyCode::N => self.register_signal_km(MethodKM::N as usize, &keyboard.state),
                            VirtualKeyCode::O => self.register_signal_km(MethodKM::O as usize, &keyboard.state),
                            VirtualKeyCode::P => self.register_signal_km(MethodKM::P as usize, &keyboard.state),
                            VirtualKeyCode::Q => self.register_signal_km(MethodKM::Q as usize, &keyboard.state),
                            VirtualKeyCode::R => self.register_signal_km(MethodKM::R as usize, &keyboard.state),
                            VirtualKeyCode::S => self.register_signal_km(MethodKM::S as usize, &keyboard.state),
                            VirtualKeyCode::T => self.register_signal_km(MethodKM::T as usize, &keyboard.state),
                            VirtualKeyCode::U => self.register_signal_km(MethodKM::U as usize, &keyboard.state),
                            VirtualKeyCode::V => self.register_signal_km(MethodKM::V as usize, &keyboard.state),
                            VirtualKeyCode::W => self.register_signal_km(MethodKM::W as usize, &keyboard.state),
                            VirtualKeyCode::X => self.register_signal_km(MethodKM::X as usize, &keyboard.state),
                            VirtualKeyCode::Y => self.register_signal_km(MethodKM::Y as usize, &keyboard.state),
                            VirtualKeyCode::Z => self.register_signal_km(MethodKM::Z as usize, &keyboard.state),

                            _ => panic!("Virtual key code can not be converted: {0}",  active as u32)
                        }
                    }
                    _ => ()
                }

            }
            DeviceEvent::Button { button, state } =>
            {
                match button
                {
                    1 => self.register_signal_km(MethodKM::MouseLeft as usize, state),
                    2 => self.register_signal_km(MethodKM::MouseMiddle as usize, state),
                    3 => self.register_signal_km(MethodKM::MouseRight as usize, state),

                    _ => panic!("Button ID can not be converted: {0}",  *button as u32)
                }
            }
            _ => ()
        }
    }

    fn register_signal_km
    (
        &mut self,
        slot: usize,
        state: &ElementState
    )
    {
        const ALWAYS_ONE: f32 = 1.0;

        self.sps.count(false);
    
        match state
        {
            ElementState::Pressed =>
                {
                    match self.signals_km[slot]
                    {
                        Signal::Inactive => self.signals_km[slot] = Signal::Active(ALWAYS_ONE, CheckedOnce(false)),
                        Signal::Active(_, checked_once) => self.signals_km[slot] = Signal::Active(ALWAYS_ONE, CheckedOnce(checked_once.0)),
                    }
                }
            ElementState::Released => self.signals_km[slot] = Signal::Inactive
        }
    }

    fn register_signal_gp
    (
        &mut self,
        slot: usize,
        signal: Signal
    )
    {
        self.sps.count(false);
        unimplemented!()
    }

    pub fn register_signal_mouse_cursor
    (
        &mut self,
        position: [f64; 2],
        window_size: [u32; 2]
    )
    {
        self.sps.count(false);

        // Also convert from Top-Left origin to Center origin.
        self.signals_mouse_cursor = [position[0] as f32 - window_size[0] as f32 / 2.0, position[1] as f32 - window_size[1] as f32 / 2.0];
        //println!("X {0}, Y {1}", self.signals_mouse_cursor[0], self.signals_mouse_cursor[1])
    }

    fn index_to_string
    (
        slot: usize,
        methodkm: MethodKM,
        string_add: &str,
        string: &mut String
    )
    {
        if slot == methodkm as usize
        {
            *string += string_add;
        }
    }

    pub fn print_mappings
    (
        &self
    )
    {
        println!();
        println!("List of input mappings:");
        for (index, mapping) in self.mappings.all().iter().enumerate()
        {
            let mapping = mapping.read().unwrap();

            println!();
            println!("\t[{}] {}", index, mapping.name);

            match mapping.custom
            {
                Some(custom) => 
                {
                    //println!();
                    //println!("[{}] {}", index, mapping.name);
                    println!("\tCustom: {}", custom.make_str());
                    println!("\tDefault: {}", mapping.default.make_str())
                }
                None => 
                {
                    //println!("\t'{}' with default '{}'.", mapping.name, mapping.default.make_str())
                    println!("\tDefault: {}", mapping.default.make_str())
                }
            };            
        }
    }

    pub fn default_mappings
    (
        &mut self
    )
    {        
        for mapping in self.mappings.all().iter()
        {
            let mut mapping = mapping.write().unwrap();
            mapping.custom = None;
        }
        println!("All mappings set to default.");
    }
}

impl SystemEvents for InputSystem
{
    fn console
    (
        &mut self,
        framework: &mut Framework
    )
    {
        let command = ConsoleCommand::new("imaps", Vec::new());        
        if framework.command_event() == &command
        {
            self.print_mappings();
        }

        let command = ConsoleCommand::new("ibind", vec![ConsoleCommandParameter::U32]);
        if framework.command_event() == &command
        {
            unimplemented!();
        }
    }

    fn save_load
    (
        &mut self,
        framework: &mut crate::Framework
    )
    {
        println!("Saved.")    
    }
}

#[derive(Clone)]
pub struct Mapping
{
    name: String,
    default: MethodKM,
    custom: Option<MethodKM>,
    //Typing,
    //HotKey
}

impl Mapping
{
    pub fn new
    (
        name: &str,
        default: MethodKM
    )
    -> Mapping
    {
        Mapping { name: name.to_string(), default, custom: None }
    }

    pub fn name
    (
        self
    )
    -> String
    {
        self.name
    }

    pub fn bind_custom
    (
        &mut self,
        mapping: MethodKM
    )
    {
        self.custom = Some(mapping);
    }
}

pub enum Mode
{
    Normal,

    /// Only accepts Numbers, Lower case letters, Space.
    TypingNLS,
    HotKey
}

const MAX_SIGNAL_SLOTS_KM: usize = 58;

/// Input bindings for the mouse and keyboard input method.
enum_str!
{
    #[derive(Copy, Clone)]
    pub enum MethodKM // TODO List needs finishing.
    {
        ESC,
        TAB,
        Space,
        Backspace,
        Enter,
        ShiftLeft,
        ShiftRight,

        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,

        Key0,
        Key1,
        Key2,
        Key3,
        Key4,
        Key5,
        Key6,
        Key7,
        Key8,
        Key9,

        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        N,
        M,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,

        MouseLeft,
        MouseMiddle,
        MouseRight,
    }
}

const MAX_SIGNAL_SLOTS_GP: usize = 4;

/// Input bindings for the gamepad input method.
pub enum MethodGP // todo List needmappings finishing.
{
    ButtonAX,
    ButtonBCircle,
    ButtonXRect,
    ButtonYTriangle
}

#[derive(Copy, Clone)]
pub struct CheckedOnce(pub(crate) bool);

#[derive(Copy, Clone)]
pub enum Signal
{
    Inactive,
    Active(f32, CheckedOnce)
}