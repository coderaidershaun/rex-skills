// Public document API surface, up for review before we publish the crate.

pub mod parser {
    pub struct ParserConfig {
        pub strict: bool,
        pub max_depth: u32,
    }

    pub struct ParserResult {
        pub documents: Vec<super::Document>,
        pub skipped: u32,
    }

    pub fn parser_run(config: &ParserConfig, input: &str) -> ParserResult {
        let documents = input
            .split("\n---\n")
            .filter(|chunk| !config.strict || !chunk.is_empty())
            .map(|chunk| super::Document::new(chunk))
            .collect();
        ParserResult { documents, skipped: 0 }
    }
}

pub struct Document {
    body: String,
    tags: Vec<String>,
}

impl Document {
    pub fn new(body: &str) -> Self {
        Self { body: body.to_string(), tags: Vec::new() }
    }

    pub fn get_body(&self) -> &str {
        &self.body
    }

    pub fn as_summary(&self) -> String {
        let end = self.body.len().min(40);
        self.body[..end].to_string()
    }

    pub fn to_len(&self) -> usize {
        self.body.len()
    }

    pub fn into_tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    pub fn iterate(&self) -> std::slice::Iter<'_, String> {
        self.tags.iter()
    }

    pub fn push_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }
}
