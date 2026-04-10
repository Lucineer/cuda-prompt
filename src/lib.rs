/*!
# cuda-prompt

Prompt engineering primitives.

The right prompt is half the battle. This crate gives agents tools to
compose, template, optimize, and manage prompts systematically instead
of ad-hoc string manipulation.

- Template composition with variable injection
- Chain-of-thought scaffolding
- Prompt versioning and A/B tracking
- Token estimation
- Prompt compression
- Few-shot management
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A prompt template with variable slots
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub raw: String,
    pub variables: Vec<String>,
    pub version: u32,
    pub category: String,
}

impl PromptTemplate {
    pub fn new(name: &str, raw: &str) -> Self {
        let variables = Self::extract_vars(raw);
        PromptTemplate { name: name.to_string(), raw: raw.to_string(), variables, version: 1, category: String::new() }
    }

    /// Extract {{variable}} names from template
    fn extract_vars(template: &str) -> Vec<String> {
        let mut vars = vec![];
        let bytes = template.as_bytes();
        let mut i = 0;
        while i + 2 < bytes.len() {
            if bytes[i] == b'{' && bytes[i+1] == b'{' {
                let end = template[i+2..].find("}}").map(|e| i + 2 + e).unwrap_or(template.len());
                vars.push(template[i+2..end].trim().to_string());
                i = end + 2;
            } else { i += 1; }
        }
        vars
    }

    /// Render template with variable values
    pub fn render(&self, values: &HashMap<String, String>) -> String {
        let mut result = self.raw.clone();
        for var in &self.variables {
            let placeholder = format!("{{{{{}}}}}", var);
            let replacement = values.get(var).cloned().unwrap_or_else(|| format!("[{}]", var));
            result = result.replace(&placeholder, &replacement);
        }
        result
    }

    /// Estimate token count (rough: ~4 chars per token)
    pub fn estimate_tokens(&self) -> usize { (self.raw.len() / 4).max(1) }
}

/// A chain-of-thought step
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CotStep {
    pub instruction: String,
    pub label: String,
    pub pause_after: bool, // stop after this step for human review
}

/// Chain-of-thought scaffold
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CotScaffold {
    pub steps: Vec<CotStep>,
    pub separator: String,
}

impl CotScaffold {
    pub fn new() -> Self { CotScaffold { steps: vec![], separator: "\n\n---\n\n".into() } }

    pub fn add_step(&mut self, instruction: &str, label: &str) { self.steps.push(CotStep { instruction: instruction.to_string(), label: label.to_string(), pause_after: false }); }
    pub fn add_pause(&mut self, instruction: &str, label: &str) { self.steps.push(CotStep { instruction: instruction.to_string(), label: label.to_string(), pause_after: true }); }

    pub fn render(&self) -> String {
        self.steps.iter().map(|s| {
            if s.pause_after { format!("[PAUSE: {}]\n{}", s.label, s.instruction) }
            else { format!("[Step: {}]\n{}", s.label, s.instruction) }
        }).collect::<Vec<_>>().join(&self.separator)
    }
}

/// A few-shot example
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FewShotExample {
    pub input: String,
    pub output: String,
    pub label: String,
}

/// Prompt version for A/B testing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptVersion {
    pub id: String,
    pub template: PromptTemplate,
    pub score: f64,       // performance score 0-1
    pub uses: u64,
    pub successes: u64,
    pub created: u64,
}

impl PromptVersion {
    pub fn success_rate(&self) -> f64 { if self.uses == 0 { return 0.0; } self.successes as f64 / self.uses as f64 }
    pub fn record_use(&mut self, success: bool) { self.uses += 1; if success { self.successes += 1; } }
}

/// The prompt manager
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptManager {
    pub templates: HashMap<String, PromptTemplate>,
    pub versions: HashMap<String, Vec<PromptVersion>>,
    pub few_shots: HashMap<String, Vec<FewShotExample>>,
    pub default_separators: HashMap<String, String>,
}

impl PromptManager {
    pub fn new() -> Self { PromptManager { templates: HashMap::new(), versions: HashMap::new(), few_shots: HashMap::new(), default_separators: HashMap::new() } }

    /// Register a template
    pub fn register(&mut self, template: PromptTemplate) {
        let name = template.name.clone();
        self.templates.insert(name.clone(), template);
    }

    /// Render a template
    pub fn render(&self, name: &str, values: &HashMap<String, String>) -> Option<String> {
        self.templates.get(name).map(|t| t.render(values))
    }

    /// Compose multiple templates sequentially
    pub fn compose(&self, names: &[&str], values: &HashMap<String, String>, separator: &str) -> String {
        names.iter()
            .filter_map(|n| self.render(n, values))
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// Add few-shot examples
    pub fn add_few_shot(&mut self, template_name: &str, example: FewShotExample) {
        self.few_shots.entry(template_name.to_string()).or_insert_with(Vec::new).push(example);
    }

    /// Build prompt with few-shots prepended
    pub fn render_with_shots(&self, name: &str, values: &HashMap<String, String>, max_shots: usize) -> Option<String> {
        let mut prompt = String::new();
        if let Some(shots) = self.few_shots.get(name) {
            for shot in shots.iter().take(max_shots) {
                prompt.push_str(&format!("Input: {}\nOutput: {}\n\n", shot.input, shot.output));
            }
        }
        self.render(name, values).map(|p| { prompt.push_str(&p); prompt })
    }

    /// Create a new version of a template
    pub fn version_template(&mut self, name: &str, new_raw: &str) -> Option<String> {
        let base = self.templates.get(name)?;
        let mut new_template = PromptTemplate::new(&format!("{}_v{}", name, base.version + 1), new_raw);
        new_template.version = base.version + 1;
        new_template.category = base.category.clone();
        let id = format!("{}_{}", name, base.version + 1);
        self.versions.entry(name.to_string()).or_insert_with(Vec::new)
            .push(PromptVersion { id: id.clone(), template: new_template, score: 0.5, uses: 0, successes: 0, created: now() });
        Some(id)
    }

    /// Get best version by score
    pub fn best_version(&self, name: &str) -> Option<&PromptVersion> {
        self.versions.get(name)?.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Simple compression: remove redundant whitespace, collapse repeated newlines
    pub fn compress(prompt: &str) -> String {
        let mut result = String::with_capacity(prompt.len());
        let mut prev_newline = false;
        let mut prev_space = false;
        for ch in prompt.chars() {
            if ch == '\n' {
                if prev_newline { continue; }
                prev_newline = true; prev_space = false;
                result.push(ch);
            } else if ch == ' ' {
                if prev_space { continue; }
                prev_space = true; prev_newline = false;
                result.push(ch);
            } else {
                prev_newline = false; prev_space = false;
                result.push(ch);
            }
        }
        result
    }

    /// Estimate tokens for a rendered string
    pub fn estimate_tokens(text: &str) -> usize { (text.len() / 4).max(1) }

    /// Summary
    pub fn summary(&self) -> String {
        format!("PromptManager: {} templates, {} versioned, {} few-shot sets",
            self.templates.len(), self.versions.len(), self.few_shots.len())
    }
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_render() {
        let t = PromptTemplate::new("greet", "Hello {{name}}, you are a {{role}}.");
        let mut vars = HashMap::new();
        vars.insert("name".into(), "Casey".into());
        vars.insert("role".into(), "Captain".into());
        let rendered = t.render(&vars);
        assert_eq!(rendered, "Hello Casey, you are a Captain.");
    }

    #[test]
    fn test_missing_variable() {
        let t = PromptTemplate::new("x", "Value: {{val}}");
        let rendered = t.render(&HashMap::new());
        assert!(rendered.contains("[val]"));
    }

    #[test]
    fn test_extract_vars() {
        let t = PromptTemplate::new("x", "{{a}} and {{b}} but not {{a}}");
        assert_eq!(t.variables, vec!["a", "b", "a"]);
    }

    #[test]
    fn test_cot_scaffold() {
        let mut cot = CotScaffold::new();
        cot.add_step("Think about the problem", "think");
        cot.add_step("Write the solution", "solve");
        let rendered = cot.render();
        assert!(rendered.contains("[Step: think]"));
        assert!(rendered.contains("[Step: solve]"));
    }

    #[test]
    fn test_few_shot_rendering() {
        let mut pm = PromptManager::new();
        pm.register(PromptTemplate::new("classify", "Classify: {{text}}"));
        pm.add_few_shot("classify", FewShotExample { input: "hello".into(), output: "greeting".into(), label: "ex1".into() });
        let result = pm.render_with_shots("classify", &HashMap::new(), 5);
        assert!(result.unwrap().contains("Input: hello"));
        assert!(result.unwrap().contains("Classify:"));
    }

    #[test]
    fn test_compose() {
        let mut pm = PromptManager::new();
        pm.register(PromptTemplate::new("sys", "System: {{sys_msg}}"));
        pm.register(PromptTemplate::new("user", "User: {{user_msg}}"));
        let mut vars = HashMap::new();
        vars.insert("sys_msg".into(), "You are helpful".into());
        vars.insert("user_msg".into(), "Hi".into());
        let composed = pm.compose(&["sys", "user"], &vars, "\n---\n");
        assert!(composed.contains("System: You are helpful"));
        assert!(composed.contains("User: Hi"));
    }

    #[test]
    fn test_versioning() {
        let mut pm = PromptManager::new();
        pm.register(PromptTemplate::new("prompt", "v1: {{x}}"));
        let v2_id = pm.version_template("prompt", "v2: {{x}} with detail");
        assert!(v2_id.is_some());
        assert_eq!(pm.versions.get("prompt").unwrap().len(), 1);
    }

    #[test]
    fn test_version_scoring() {
        let mut pm = PromptManager::new();
        pm.register(PromptTemplate::new("p", "x"));
        pm.version_template("p", "v2");
        if let Some(versions) = pm.versions.get_mut("p") {
            versions[0].record_use(true);
            versions[0].record_use(true);
            versions[0].record_use(false);
        }
        let best = pm.best_version("p").unwrap();
        assert!((best.success_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_compress() {
        let input = "Hello    world\n\n\nExtra   spaces";
        let compressed = PromptManager::compress(input);
        assert!(!compressed.contains("    "));
        assert!(!compressed.contains("\n\n\n"));
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(PromptManager::estimate_tokens("a"), 1);
        assert!(PromptManager::estimate_tokens("hello world this is a test") > 2);
    }
}
