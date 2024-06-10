use crate::bit_set::*;

pub type Domain = BitSet;
pub type Domains = Vec<Domain>;

pub type Variable = usize;
pub type VariableSet = BitSet;
pub type ConstraintID = usize;

pub struct Reporter {
    pub variable_id_to_name: Vec<String>,
    pub constraint_id_to_name: Vec<String>,
    pub enabled: bool,
}

impl Reporter {
    pub fn variable_name(&self, id: Variable) -> &String {
        &self.variable_id_to_name[id]
    }

    pub fn constraint_name(&self, id: ConstraintID) -> &String {
        &self.constraint_id_to_name[id]
    }

    pub fn emit(&self, breadcrumb: String) {
        eprint!("{}\n", breadcrumb);
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone,Copy)]
pub struct Config {
    pub greedy: bool,
    pub max_depth: u64,
}
