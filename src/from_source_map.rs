use serde_json;
use source_map_mappings::{parse_mappings, Mappings as _Mappings};
use source_map_generator::SourceMapGenerator;
use source_map::SrcMap;
use mapping::Mapping;
use std::collections::HashSet;
use linked_hash_map::LinkedHashMap;
use std::rc::Rc;
use std::hash::Hash;
use StringPtr;

pub fn from_source_map(source_map: &str, check_dup: bool) -> SourceMapGenerator {
    let source_map: SrcMap = serde_json::from_str(source_map).unwrap();
    let file = source_map.file.map(|s| StringPtr::Str(s));
    let source_root = source_map.source_root.map(|s| StringPtr::Str(s));

    let mut generator = SourceMapGenerator::new(file, source_root, true);

    let mut contents = source_map.sources_content.into_iter().map(|s| StringPtr::Ptr(Rc::new(s)));
    let sources: Vec<Rc<String>> = if check_dup {
        let mut set: HashSet<Rc<String>> = HashSet::new();
        source_map.sources.into_iter().map(|s| Rc::new(s)).filter(|sp| {
            generator.set_source_content(StringPtr::Ptr(sp.clone()), contents.next());
            set.insert(sp.clone())
        }).collect()
    } else {
        source_map.sources.into_iter().map(|s| {
            let sp = Rc::new(s);
            generator.set_source_content(StringPtr::Ptr(sp.clone()), contents.next());
            sp
        }).collect()
    };
    let names: Vec<Rc<String>> = if check_dup {
        let mut set: HashSet<Rc<String>> = HashSet::new();
        source_map.names.into_iter().map(|s| Rc::new(s)).filter(|sp|
            set.insert(sp.clone())
        ).collect()
    } else {
        source_map.names.into_iter().map(|s| Rc::new(s)).collect()
    };

    let mappings: _Mappings<()> = parse_mappings(source_map.mappings.as_bytes()).unwrap();
    let mappings = mappings.by_generated_location();

    for mapping in mappings {
        let generated = (mapping.generated_line as usize, mapping.generated_column as usize);
        let (original, source, name) = if let Some(original) = mapping.original.clone() {
            let name = original.name.map(|idx| names[idx as usize].clone());
            let source = sources[original.source as usize].clone();
            (Some((original.original_line as usize, original.original_column as usize)), Some(source), name)
        } else {
            (None, None, None)
        };
        generator.add_mapping(Mapping {
            generated,
            original,
            source,
            name,
        })
    }
    generator
}

fn remove_dup(vec: Vec<String>) -> Vec<Rc<String>> {
    let mut new: Vec<Rc<String>> = Vec::new();
    let mut set: HashSet<Rc<String>> = HashSet::new();

    for i in vec.into_iter() {
        let s = Rc::new(i);
        if !set.contains(&s) {
            new.push(s.clone());
            set.insert(s);
        }
    }
    new
}
