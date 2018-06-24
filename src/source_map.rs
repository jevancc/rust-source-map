#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StringWithSrcMap {
    pub source: String,
    pub map: SrcMap,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SrcMap {
    pub version: i32,
    pub file: Option<String>,
    pub source_root: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
}
