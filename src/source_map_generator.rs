use linked_hash_map::LinkedHashMap;
use mapping::Mapping;
use mapping_list::MappingList;
use source_map::SrcMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::str;
use utils;
use vlq;
use StringPtr;

#[derive(Debug)]
pub struct SourceMapGenerator {
    file: Option<Rc<String>>,
    source_root: Option<Rc<String>>,
    skip_validation: bool,
    sources: LinkedHashMap<Rc<String>, usize>,
    names: LinkedHashMap<Rc<String>, usize>,
    mappings: MappingList,
    sources_contents: HashMap<Rc<String>, Rc<String>>,
}

impl SourceMapGenerator {
    pub fn new(
        file: Option<StringPtr>,
        source_root: Option<StringPtr>,
        skip_validation: bool,
    ) -> SourceMapGenerator {
        let file = file.map(|sp| sp.to_ptr());
        let source_root = source_root.map(|sp| sp.to_ptr());
        SourceMapGenerator {
            file,
            source_root,
            skip_validation,
            sources: LinkedHashMap::new(),
            names: LinkedHashMap::new(),
            mappings: MappingList::new(),
            sources_contents: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, map: Mapping) {
        if !self.skip_validation {
            SourceMapGenerator::validate_mapping(&map).unwrap();
        }

        if let Some(source) = map.source.clone() {
            let len = self.sources.len();
            self.sources.entry(source).or_insert(len);
        }

        if let Some(name) = map.name.clone() {
            let len = self.names.len();
            self.names.entry(name).or_insert(len);
        }

        self.mappings.add(map);
    }

    pub fn set_source_content(
        &mut self,
        source_file: StringPtr,
        source_content: Option<StringPtr>,
    ) {
        let source_file = source_file.to_ptr();
        let source_content = source_content.map(|sp| sp.to_ptr());

        let source = if let Some(root) = self.source_root.clone() {
            Rc::new(utils::relative(&root, &source_file))
        } else {
            source_file
        };

        if let Some(content) = source_content {
            self.sources_contents.entry(source).or_insert(content);
        } else {
            self.sources_contents.remove(&source);
        }
    }

    pub fn to_source_map(&mut self) -> SrcMap {
        let version = 3;
        let sources: Vec<String> = self.sources.keys().map(|sp| (**sp).clone()).collect();
        let names: Vec<String> = self.names.keys().map(|sp| (**sp).clone()).collect();
        let mappings = self.serialize_mappings();
        let file = self.file.clone().map(|sp| (*sp).clone());
        let source_root = self.source_root.clone().map(|sp| (*sp).clone());
        let mut sources_content: Vec<String> = Vec::new();

        for src in self.sources.keys() {
            if let Some(content) = self.sources_contents.get(src) {
                sources_content.push((**content).clone());
            }
        }
        return SrcMap {
            version,
            sources,
            names,
            mappings,
            file,
            source_root,
            sources_content,
        };
    }

    fn validate_mapping(map: &Mapping) -> Result<(), &'static str> {
        if let Some((original_line, _)) = map.original.clone() {
            if map.source.is_some() && original_line > 0 && map.generated.0 > 0 {
                Ok(())
            } else {
                Err("Invalid mapping")
            }
        } else {
            if map.source.is_none() && map.name.is_none() && map.generated.0 > 0 {
                Ok(())
            } else {
                Err("Invalid mapping")
            }
        }
    }

    fn serialize_mappings(&mut self) -> String {
        // (line, column)
        let mut previous_generated: (usize, usize) = (1, 0);
        let mut previous_original: (usize, usize) = (0, 0);
        let mut previous_name: usize = 0;
        let mut previous_source: usize = 0;
        let mut result = String::new();
        let mut buf = Vec::<u8>::new();

        self.mappings.sort();
        for (i, mapping) in self.mappings.list.iter().enumerate() {
            if mapping.generated.0 != previous_generated.0 {
                previous_generated.1 = 0;
                for _ in 0..(mapping.generated.0 - previous_generated.0) {
                    buf.push(b';');
                }
                previous_generated.0 = mapping.generated.0;
            } else if i > 0 {
                //     if (
                //         !util.compareByGeneratedPositionsInflated(
                //             mapping,
                //             mappings[i - 1]
                //         )
                //     ) {
                //         continue;
                //     }
                // }
                buf.push(b',');
            }

            vlq::encode(
                mapping.generated.1 as i64 - previous_generated.1 as i64,
                &mut buf,
            ).unwrap();
            previous_generated.1 = mapping.generated.1;
            if let Some(ref source) = mapping.source {
                let source_idx = self.sources.get(source).unwrap();
                vlq::encode(*source_idx as i64 - previous_source as i64, &mut buf).unwrap();
                previous_source = *source_idx;

                let mapping_original = mapping.original.unwrap();
                // lines are stored 0-based in SourceMap spec version 3
                vlq::encode(
                    mapping_original.0 as i64 - 1 - previous_original.0 as i64,
                    &mut buf,
                ).unwrap();
                previous_original.0 = mapping_original.0 - 1;

                vlq::encode(
                    mapping_original.1 as i64 - previous_original.1 as i64,
                    &mut buf,
                ).unwrap();
                previous_original.1 = mapping_original.1;

                if let Some(ref name) = mapping.name {
                    let name_idx = self.names.get(name).unwrap();
                    vlq::encode(*name_idx as i64 - previous_name as i64, &mut buf).unwrap();
                    previous_name = *name_idx;
                }
            }
            result += str::from_utf8(&buf).unwrap();
            buf.clear();
        }
        result
    }
}
