use crate::constraint::*;
use crate::types::*;

#[derive(Clone,Debug)]
pub struct Ratio {
    id: ConstraintID,
    variables: VariableSet,
    ratio: usize,
}

impl Ratio {

    pub fn new(id: ConstraintID, variables: VariableSet, ratio: usize) -> Self {
        if variables.len() != 2 || ratio == 0 {
            panic!("bad Ratio")
        }
        return Ratio {
            id,
            variables,
            ratio,
        };
    }

}

// TODO could just multiply the bits as a number?
fn ratio_image(domain: Domain, ratio: usize) -> Domain {
    let mut image = Domain::new();
    for value in domain.iter() {
        match value.checked_mul(ratio) {
            Some(x) => { image.insert(x); }
            _ => {}
        }
        if value % ratio == 0 {
            image.insert(value / ratio);
        }
    }
    return image;
}

impl Constraint for Ratio {

    fn clone_box(&self) -> Box<dyn Constraint> { Box::new(self.clone()) }

    fn check_solved(&self, domains: &mut Domains) -> bool {

        let mut iter = self.variables.iter();
        let v1 = iter.next().unwrap();
        let v2 = iter.next().unwrap();

        let d1 = domains[v1].value_unchecked();
        let d2 = domains[v2].value_unchecked();

        return d1.checked_mul(self.ratio) == Some(d2) || d2.checked_mul(self.ratio) == Some(d1);
    }

    fn simplify(&self, domains: &mut Domains, reporter: &dyn Reporter) -> SimplifyResult {

        let mut iter = self.variables.iter();
        let v1 = iter.next().unwrap();
        let v2 = iter.next().unwrap();

        let d1 = domains[v1];
        let d2 = domains[v2];

        let mut progress = false;

        {
            progress |= apply(&*self, domains, reporter, v2, |d| d.intersect_with(ratio_image(d1, self.ratio)));
            if domains[v2].empty() {
                return SimplifyResult::Stuck;
            }
        }

        {
            progress |= apply(&*self, domains, reporter, v1, |d| d.intersect_with(ratio_image(d2, self.ratio)));
            if domains[v1].empty() {
                return SimplifyResult::Stuck;
            }
        }

        if progress {
            return SimplifyResult::Progress;
        } else {
            return SimplifyResult::Stuck;
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variables
    }

    fn id(&self) -> ConstraintID {
        self.id
    }
}

