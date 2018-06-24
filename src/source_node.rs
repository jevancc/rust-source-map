use mapping::Mapping;
use source_map::StringWithSrcMap;
use source_map_generator::SourceMapGenerator;
use std::collections::HashMap;
use std::rc::Rc;
use Node;
use StringPtr;

pub struct SourceNode {
    pub children: Vec<Node>,
    pub source_contents: HashMap<Rc<String>, Rc<String>>,
    // (line, column)
    pub position: Option<(usize, usize)>,
    pub source: Option<Rc<String>>,
    pub name: Option<Rc<String>>,
}

impl SourceNode {
    pub fn new(
        position: Option<(usize, usize)>,
        source: Option<StringPtr>,
        name: Option<StringPtr>,
        chunks: Option<Node>,
    ) -> SourceNode {
        let source = source.map(|sp| sp.to_ptr());
        let name = name.map(|sp| sp.to_ptr());
        let mut sn = SourceNode {
            position,
            source,
            name,
            children: Vec::new(),
            source_contents: HashMap::new(),
        };
        if let Some(chunks) = chunks {
            sn.add(chunks);
        }
        sn
    }

    pub fn add(&mut self, chunk: Node) {
        match chunk {
            Node::NNodeVec(mut nv) => {
                self.children.append(&mut nv);
            }
            Node::NSourceNode(sn) => {
                self.children.push(Node::NSourceNode(sn));
            }
            Node::NString(s) => {
                self.children.push(Node::NRcString(Rc::new(s)));
            }
            Node::NRcString(sp) => {
                self.children.push(Node::NRcString(sp));
            }
        }
    }

    pub fn set_source_content(&mut self, source: StringPtr, source_content: StringPtr) {
        let source = source.to_ptr();
        let source_content = source_content.to_ptr();
        self.source_contents.insert(source, source_content);
    }

    pub fn to_string_with_source_map(
        &self,
        file: Option<StringPtr>,
        source_root: Option<StringPtr>,
    ) -> StringWithSrcMap {
        let file = file.map(|sp| sp.to_ptr());
        let source_root = source_root.map(|sp| sp.to_ptr());
        let skip_validation = true;
        let mut context = ToSourceMapContext::new(file, source_root, skip_validation);
        self.walk(&mut context);

        StringWithSrcMap {
            source: context.generated_code,
            map: context.map.to_source_map(),
        }
    }

    fn walk<T: WalkFunction>(&self, context: &mut T) {
        for child in &self.children {
            match child {
                Node::NSourceNode(sn) => {
                    sn.walk(context);
                }
                Node::NRcString(chunk) => {
                    context.process_chunk(&chunk, &self.source, &self.position, &self.name);
                }
                _ => {}
            }
        }
        for (source, source_content) in &self.source_contents {
            context.process_source_content(source, source_content);
        }
    }
}

struct ToSourceMapContext {
    pub map: SourceMapGenerator,
    source_mapping_active: bool,
    last_original_source: Option<Rc<String>>,
    last_original_position: Option<(usize, usize)>,
    last_original_name: Option<Rc<String>>,
    generated_code: String,
    generated_position: (usize, usize),
}

impl ToSourceMapContext {
    pub fn new(
        file: Option<Rc<String>>,
        source_root: Option<Rc<String>>,
        skip_validation: bool,
    ) -> ToSourceMapContext {
        let file = file.map(|sp| StringPtr::Ptr(sp));
        let source_root = source_root.map(|sp| StringPtr::Ptr(sp));
        ToSourceMapContext {
            map: SourceMapGenerator::new(file, source_root, skip_validation),
            source_mapping_active: false,
            last_original_source: None,
            last_original_position: None,
            last_original_name: None,
            generated_code: String::new(),
            generated_position: (1, 0),
        }
    }
}

impl WalkFunction for ToSourceMapContext {
    fn process_chunk(
        &mut self,
        chunk: &Rc<String>,
        original_source: &Option<Rc<String>>,
        original_position: &Option<(usize, usize)>,
        original_name: &Option<Rc<String>>,
    ) {
        self.generated_code += chunk;
        if original_source.is_some() && original_position.is_some() {
            if self.last_original_source != *original_source
                || self.last_original_position != *original_position
                || self.last_original_name != *original_name
            {
                self.map.add_mapping(Mapping {
                    source: original_source.clone(),
                    original: original_position.clone(),
                    generated: self.generated_position.clone(),
                    name: original_name.clone(),
                });
            }
            self.last_original_source = original_source.clone();
            self.last_original_position = original_position.clone();
            self.last_original_name = original_name.clone();
            self.source_mapping_active = true;
        } else if self.source_mapping_active {
            self.map.add_mapping(Mapping {
                source: None,
                original: None,
                generated: self.generated_position.clone(),
                name: None,
            });
            self.last_original_source = None;
            self.source_mapping_active = false;
        }
        let mut chars = chunk.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\n' {
                self.generated_position.0 += 1; // line++
                self.generated_position.1 = 0; // column = 0

                if chars.peek().is_none() {
                    self.last_original_source = None;
                    self.source_mapping_active = false;
                } else if self.source_mapping_active {
                    self.map.add_mapping(Mapping {
                        source: original_source.clone(),
                        original: original_position.clone(),
                        generated: self.generated_position.clone(),
                        name: original_name.clone(),
                    })
                }
            } else {
                self.generated_position.1 += 1; // column++
            }
        }
    }

    fn process_source_content(&mut self, source: &Rc<String>, source_content: &Rc<String>) {
        self.map.set_source_content(
            StringPtr::Ptr(source.clone()),
            Some(StringPtr::Ptr(source_content.clone())),
        );
    }
}

trait WalkFunction {
    fn process_chunk(
        &mut self,
        chunk: &Rc<String>,
        original_source: &Option<Rc<String>>,
        original_position: &Option<(usize, usize)>,
        original_name: &Option<Rc<String>>,
    );
    fn process_source_content(&mut self, source: &Rc<String>, source_content: &Rc<String>);
}
