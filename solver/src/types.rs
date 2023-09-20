use crate::bit_set::*;

pub type Domain = BitSet;
pub type Domains = Vec<Domain>;

pub type Variable = usize;
pub type VariableSet = BitSet;

pub trait Reporter {
    fn variable_name(&self, variable: Variable) -> &String;
    fn on_progress(&mut self, comment: String);
}

#[derive(Clone,Copy)]
pub struct Config {
    pub branch: bool,
    pub unique: bool,
}
