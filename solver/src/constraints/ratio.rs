use crate::constraint::*;
use crate::types::*;
use std::rc::Rc;

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

    fn check_solved(&self, domains: &mut Domains) -> bool {

        let mut iter = self.variables.iter();
        let v1 = iter.next().unwrap();
        let v2 = iter.next().unwrap();

        let d1 = domains.get(v1).unwrap().value_unchecked();
        let d2 = domains.get(v2).unwrap().value_unchecked();

        return d1.checked_mul(self.ratio) == Some(d2) || d2.checked_mul(self.ratio) == Some(d1);
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {

        let mut iter = self.variables.iter();
        let v1 = iter.next().unwrap();
        let v2 = iter.next().unwrap();

        let d1 = *domains.get(v1).unwrap();
        let d2 = *domains.get(v2).unwrap();

        let mut progress = false;

        {
            let new = domains.get_mut(v2).unwrap();
            let old = *new;
            new.intersect_with(ratio_image(d1, self.ratio));
            if *new != old {
                progress = true;
                if reporter.enabled() {
                    reporter.emit(format!("{} is not {} since {}", reporter.variable_name(v2), old.difference(*new), reporter.constraint_name(self.id)));
                }
            }
            if new.empty() {
                return Result::Stuck;
            }
        }

        {
            let new = domains.get_mut(v1).unwrap();
            let old = *new;
            new.intersect_with(ratio_image(d2, self.ratio));
            if *new != old {
                progress = true;
                if reporter.enabled() {
                    reporter.emit(format!("{} is not {} since {}", reporter.variable_name(v1), old.difference(*new), reporter.constraint_name(self.id)));
                }
            }
            if new.empty() {
                return Result::Stuck;
            }
        }

        if progress {
            return Result::Progress(vec![BoxedConstraint::new(self)]);
        } else {
            return Result::Stuck;
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variables
    }

    fn id(&self) -> ConstraintID {
        self.id
    }
}

