use super::*;

#[derive(Debug, Clone)]
pub enum Param {
    Input(Name),
    Output(Name),
    InputAndOutput(Name),
}

impl Param {
    pub fn name(&self) -> &Name {
        match self {
            &Param::Input(ref name)
            | &Param::Output(ref name)
            | &Param::InputAndOutput(ref name) => name,
        }
    }

    pub fn pre_initialised(&self) -> bool {
        match self {
            &Param::Input(_) | &Param::InputAndOutput(_) => true,
            &Param::Output(_) => false,
        }
    }

    pub fn outputted(&self) -> bool {
        match self {
            &Param::Output(_) | &Param::InputAndOutput(_) => true,
            &Param::Input(_) => false,
        }
    }
}
