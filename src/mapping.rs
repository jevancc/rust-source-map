use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Mapping {
    // (line, column)
    pub generated: (usize, usize),
    pub source: Option<Rc<String>>,
    pub name: Option<Rc<String>>,
    pub original: Option<(usize, usize)>,
}

impl Ord for Mapping {
    fn cmp(&self, other: &Mapping) -> Ordering {
        let cmp = self.generated.cmp(&other.generated);
        if cmp != Ordering::Equal {
            return cmp;
        }

        let cmp = strcmp(&self.source, &other.source);
        if cmp != Ordering::Equal {
            return cmp;
        }

        let cmp = self.original.cmp(&other.original);
        if cmp != Ordering::Equal {
            return cmp;
        }

        return strcmp(&self.name, &other.name);
    }
}

impl PartialOrd for Mapping {
    fn partial_cmp(&self, other: &Mapping) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn strcmp(s1: &Option<Rc<String>>, s2: &Option<Rc<String>>) -> Ordering {
    if s1.is_none() && s2.is_some() {
        Ordering::Greater
    } else if s2.is_none() && s1.is_some() {
        Ordering::Less
    } else {
        s1.cmp(s2)
    }
}
