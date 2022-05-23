use ash::vk::MappedMemoryRange;

use crate::input::{InputSystem, Mode, Mapping, MethodKM};
use crate::{Handle, Framework, SystemEvents};

#[derive(Copy, Clone)]
pub enum ConsoleState
{
    Opened,
    Closed,
}

pub struct ConsoleWidget
{
    state: ConsoleState,
    typing: String,
    input_open_console: Handle<Mapping>,    
    input_submit: Handle<Mapping>,
    input_info: Handle<Mapping>
}

impl ConsoleWidget
{
    pub fn new
    (
        input: &mut InputSystem
    )
    -> ConsoleWidget
    {
        let input_open_console = input.add_mapping(Mapping::new("(GUI) Toggle Console", MethodKM::F1));
        let input_info = input.add_mapping(Mapping::new("(GUI) Print All Console Commands", MethodKM::TAB));
        let input_submit = input.add_mapping(Mapping::new("(GUI) Console Submit", MethodKM::Enter)); // TODO Perhaps needs to go to a more global place.

        ConsoleWidget
        {
            state: ConsoleState::Closed,
            typing: String::new(),
            input_submit,
            input_open_console,
            input_info
        }
    }    

    pub fn update
    (
        &mut self,
        input: &mut InputSystem,
        framework: &mut Framework
    )
    -> ConsoleState
    {
        match self.state
        {
            ConsoleState::Closed =>
            {
                if input.check_once(&self.input_open_console)
                {       
                    input.mode(Mode::TypingNLS);                    
                    self.state = ConsoleState::Opened;
                }        

                self.state
            }
            ConsoleState::Opened =>
            {
                input.check_typing(&mut self.typing);   
                framework.window().set_title(&self.typing);       

                if input.check_once(&self.input_submit)
                {
                    framework.command(&self.typing);
                    self.typing.clear();
                }  

                if input.check_once(&self.input_info)
                {
                    println!();
                    println!("Console commands:");
                    framework.command("info");                    
                    //self.typing.clear();
                }  

                if input.check_once(&self.input_open_console)
                {       
                    input.mode(Mode::Normal);
                    framework.window().set_title("TITLE NEEDS FIXING YOU LAZY BUM!");
                    self.state = ConsoleState::Closed;
                }        

                self.state
            }            
        }
    }
}