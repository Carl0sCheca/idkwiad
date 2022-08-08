use std::collections::HashMap;

pub struct Input<'a> {
    actions: HashMap<&'a str, InputType>,
}

pub enum InputType {
    Keyboard {
        input: winit::event::KeyboardInput,
        active: bool,
    },
    Mouse {},
}
