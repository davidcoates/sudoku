use crate::bit_set::*;


pub type Domain = BitSet;
pub type Domains = Vec<Domain>;

pub type Variable = usize;
pub type VariableSet = BitSet;
pub type ConstraintID = usize;

pub trait Reporter {
    fn variable_name(&self, variable: Variable) -> &String;
    fn constraint_name(&self, id: ConstraintID) -> &String;
    fn emit(&self, breadcrumb: String);
    fn enabled(&self) -> bool;
}
