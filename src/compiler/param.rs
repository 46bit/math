use super::*;

pub fn classify_parameters(
    inputs: &Vec<Name>,
    outputs: &Vec<Name>,
    assign_set: HashSet<Name>,
) -> Result<Vec<Param>, Error> {
    let mut params = vec![];
    let mut params_index_by_name = HashMap::new();

    for input in inputs.clone() {
        // FIXME: Input names must be unique. Otherwise our current method of inputting a
        // list of values won't directly map to the `inputs a, b, c`. For now, avoid making
        // a duplicate parameter and it should still work.
        if !params_index_by_name.contains_key(&input) {
            params_index_by_name.insert(input.clone(), params.len());
            params.push(Param::Input(input));
        }
    }

    for output in outputs.clone() {
        // If present,
        //   convert Param::Input to Param::InputAndOutput
        //   do not alter Param::Output
        //   do not alter Param::InputAndOutput
        // If not present,
        //   append Param::Output
        if params_index_by_name.contains_key(&output) {
            let param_index = params_index_by_name[&output];
            let mut param = params.get_mut(param_index).unwrap();
            assert_eq!(param.name(), &output);
            if param.is_input() && !param.is_output() {
                *param = Param::InputAndOutput(output);
            }
        } else if assign_set.contains(&output) {
            params_index_by_name.insert(output.clone(), params.len());
            params.push(Param::Output(output));
        } else {
            return Err(Error::UnassignedOutput(output));
        }
    }

    Ok(params)
}

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

    pub fn is_input(&self) -> bool {
        match self {
            &Param::Input(_) | &Param::InputAndOutput(_) => true,
            &Param::Output(_) => false,
        }
    }

    pub fn is_output(&self) -> bool {
        match self {
            &Param::Output(_) | &Param::InputAndOutput(_) => true,
            &Param::Input(_) => false,
        }
    }

    pub fn pre_initialised(&self) -> bool {
        match self {
            &Param::Input(_) | &Param::InputAndOutput(_) => true,
            &Param::Output(_) => false,
        }
    }
}
