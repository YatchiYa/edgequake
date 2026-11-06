//! Ontology drafts from files and documents (SPEC-184).
//!
//! JSON and CSV/TSV are read directly. Any other text is read by an LLM in segments; the
//! caller owns the LLM call, this module owns the prompt, the parse and the merge.

use serde_json::Value;

use super::model::{OntologyEntityType, OntologyRelationType, OntologyTerm, WorkspaceOntology};
use crate::prompts::{normalize_entity_name, normalize_type_token};

/// Characters of free text read for a draft.
pub const MAX_DRAFT_TEXT_CHARS: usize = 120_000;
/// Characters per LLM segment.
pub const SEGMENT_CHARS: usize = 12_000;

/// How a draft was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftFormat {
    Json,
    Table,
    FreeText,
}

/// Read JSON (an ontology object, or a list of terms) or a CSV/TSV table with a header.
/// `Ok(None)` means the text is neither: use the LLM path.
pub fn parse_structured(
    text: &str,
    file_name: Option<&str>,
) -> Result<Option<(WorkspaceOntology, DraftFormat)>, String> {
    let trimmed = text.trim_start_matches('\u{feff}').trim();
    let ext = file_name
        .and_then(|n| n.rsplit_once('.'))
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    if ext == "json" || trimmed.starts_with('{') || trimmed.starts_with('[') {
        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => return ontology_from_json(value).map(|o| Some((o, DraftFormat::Json))),
            Err(e) if ext == "json" => return Err(format!("The file is not valid JSON: {e}")),
            Err(_) => {}
        }
    }
    if let Some(o) = ontology_from_table(trimmed) {
        return Ok(Some((o, DraftFormat::Table)));
    }
    if matches!(ext.as_str(), "csv" | "tsv") {
        return Err(
            "The table needs a header row with a term column (name, term, terme, nom…).".into(),
        );
    }
    Ok(None)
}

fn ontology_from_json(value: Value) -> Result<WorkspaceOntology, String> {
    match value {
        Value::Object(ref map)
            if map.contains_key("entity_types") || map.contains_key("terms") || map.contains_key("relation_types") =>
        {
            serde_json::from_value(value).map_err(|e| format!("The JSON is not an ontology: {e}"))
        }
        Value::Array(items) => {
            let terms: Vec<OntologyTerm> = items
                .into_iter()
                .filter_map(|item| match item {
                    Value::String(name) => Some(OntologyTerm { name, ..Default::default() }),
                    other => serde_json::from_value(other).ok(),
                })
                .collect();
            if terms.is_empty() {
                return Err("The JSON list holds no terms (objects with a \"name\").".into());
            }
            Ok(WorkspaceOntology { terms, ..Default::default() })
        }
        _ => Err("The JSON must be an ontology object (entity_types, relation_types, terms) or a list of terms.".into()),
    }
}

/// Minimal RFC 4180 reader: quoted fields, doubled quotes, newlines inside quotes.
fn read_rows(text: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
        } else if c == '"' && field.trim().is_empty() {
            field.clear();
            quoted = true;
        } else if c == delimiter {
            row.push(std::mem::take(&mut field).trim().to_string());
        } else if c == '\n' || c == '\r' {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            row.push(std::mem::take(&mut field).trim().to_string());
            if row.iter().any(|f| !f.is_empty()) {
                rows.push(std::mem::take(&mut row));
            } else {
                row.clear();
            }
        } else {
            field.push(c);
        }
    }
    row.push(field.trim().to_string());
    if row.iter().any(|f| !f.is_empty()) {
        rows.push(row);
    }
    rows
}

fn header_role(cell: &str) -> Option<&'static str> {
    let c = normalize_type_token(cell).to_lowercase().replace('_', "");
    match c.as_str() {
        "name" | "term" | "terme" | "nom" | "label" | "libelle" | "libellé" | "canonical"
        | "canonicalname" | "entity" | "entite" | "entité" => Some("name"),
        "type" | "entitytype" | "category" | "categorie" | "catégorie" | "class" | "classe"
        | "kind" => Some("type"),
        "aliases" | "alias" | "synonyms" | "synonym" | "synonymes" | "synonyme" | "variants"
        | "variantes" | "acronym" | "acronyme" | "acronymes" => Some("aliases"),
        "description" | "definition" | "définition" | "comment" | "commentaire" => {
            Some("description")
        }
        "parent" | "parenttype" | "broader" | "parentclass" => Some("parent"),
        _ => None,
    }
}

fn ontology_from_table(text: &str) -> Option<WorkspaceOntology> {
    let first_line = text.lines().next()?;
    let delimiter = [
        ('\t', first_line.matches('\t').count()),
        (';', first_line.matches(';').count()),
        (',', first_line.matches(',').count()),
    ]
    .into_iter()
    .max_by_key(|(_, n)| *n)
    .filter(|(_, n)| *n > 0)?
    .0;
    let rows = read_rows(text, delimiter);
    let header: Vec<Option<&str>> = rows.first()?.iter().map(|h| header_role(h)).collect();
    let col = |role: &str| header.iter().position(|r| *r == Some(role));
    let name_col = col("name")?;
    let (type_col, alias_col, desc_col, parent_col) = (
        col("type"),
        col("aliases"),
        col("description"),
        col("parent"),
    );

    let mut ontology = WorkspaceOntology::default();
    for row in rows.iter().skip(1) {
        let cell = |c: Option<usize>| {
            c.and_then(|i| row.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
        };
        let Some(name) = cell(Some(name_col)) else {
            continue;
        };
        let entity_type = cell(type_col)
            .map(normalize_type_token)
            .filter(|t| !t.is_empty());
        if let Some(t) = &entity_type {
            if !ontology.entity_types.iter().any(|e| &e.id == t) {
                ontology.entity_types.push(OntologyEntityType {
                    id: t.clone(),
                    parent: cell(parent_col).map(normalize_type_token),
                    ..Default::default()
                });
            }
        }
        ontology.terms.push(OntologyTerm {
            name: name.to_string(),
            entity_type,
            aliases: cell(alias_col)
                .map(|a| {
                    a.split(['|', ';', ','])
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            description: cell(desc_col).unwrap_or_default().to_string(),
        });
    }
    (!ontology.terms.is_empty()).then_some(ontology)
}

/// Split free text into segments on paragraph (or line) boundaries.
pub fn segments(text: &str) -> (Vec<String>, bool) {
    let truncated = text.chars().count() > MAX_DRAFT_TEXT_CHARS;
    let text: String = text.chars().take(MAX_DRAFT_TEXT_CHARS).collect();
    let mut out = Vec::new();
    let mut current = String::new();
    for para in text.split_inclusive('\n') {
        if current.chars().count() + para.chars().count() > SEGMENT_CHARS
            && !current.trim().is_empty()
        {
            out.push(std::mem::take(&mut current));
        }
        if para.chars().count() > SEGMENT_CHARS {
            let chars: Vec<char> = para.chars().collect();
            for piece in chars.chunks(SEGMENT_CHARS) {
                out.push(piece.iter().collect());
            }
            continue;
        }
        current.push_str(para);
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    (out, truncated)
}

pub const DRAFT_SYSTEM_PROMPT: &str = r#"You read an organization's reference material (ontology, glossary, nomenclature, list of systems, parts, programs, sites or document types) and turn it into a knowledge-graph ontology.

Return ONLY one JSON object, no prose, with this shape:
{
  "name": "short name of the ontology",
  "entity_types": [{"id": "UPPER_SNAKE", "label": "human label", "description": "one-sentence definition", "parent": "PARENT_ID or null", "synonyms": ["OTHER_NAME"]}],
  "relation_types": [{"id": "UPPER_SNAKE", "description": "one-sentence definition", "source_types": ["TYPE_ID"], "target_types": ["TYPE_ID"]}],
  "terms": [{"name": "canonical name as written in the material", "entity_type": "TYPE_ID", "aliases": ["acronym", "other spelling"], "description": "short definition"}]
}

Rules:
- Only what the material states or clearly defines. Do not invent terms, types or relations.
- entity_types are kinds of things (e.g. ENGINE, COMPONENT, SUPPLIER), not individual things. Use a parent when the material gives a hierarchy.
- terms are individual named things (a part number, a program, a system, a site, an acronym with its expansion). Put acronyms and spelling variants in aliases.
- Reuse the type ids already found when they fit.
- Keep the language of the material for names and descriptions; ids are English-like UPPER_SNAKE.
- Empty lists are fine when the segment holds nothing of a kind."#;

/// User prompt for one segment.
pub fn draft_user_prompt(
    segment: &str,
    index: usize,
    total: usize,
    known_types: &[String],
) -> String {
    let known = if known_types.is_empty() {
        String::new()
    } else {
        format!("Type ids already found: {}\n\n", known_types.join(", "))
    };
    format!(
        "{known}Segment {} of {total}:\n\"\"\"\n{segment}\n\"\"\"\n\nJSON:",
        index + 1
    )
}

/// Parse the model's answer for one segment (tolerates code fences and prose around it).
pub fn parse_draft_response(response: &str) -> Result<WorkspaceOntology, String> {
    let start = response
        .find('{')
        .ok_or("The model returned no JSON object.")?;
    let end = response
        .rfind('}')
        .ok_or("The model returned no JSON object.")?;
    if end <= start {
        return Err("The model returned no JSON object.".into());
    }
    let value: Value = serde_json::from_str(&response[start..=end])
        .map_err(|e| format!("The model returned invalid JSON: {e}"))?;
    ontology_from_json(value)
}

/// Fold one segment's draft into the accumulated draft (first definition wins, aliases union).
pub fn merge_draft(into: &mut WorkspaceOntology, part: WorkspaceOntology) {
    if into.name.trim().is_empty() {
        into.name = part.name;
    }
    for t in part.entity_types {
        let id = normalize_type_token(if t.id.is_empty() { &t.label } else { &t.id });
        match into
            .entity_types
            .iter_mut()
            .find(|e| normalize_type_token(&e.id) == id)
        {
            Some(e) => {
                e.synonyms.extend(t.synonyms);
                if e.description.is_empty() {
                    e.description = t.description;
                }
                if e.parent.is_none() {
                    e.parent = t.parent;
                }
            }
            None => into.entity_types.push(t),
        }
    }
    for r in part.relation_types {
        let id = normalize_type_token(&r.id);
        match into
            .relation_types
            .iter_mut()
            .find(|e| normalize_type_token(&e.id) == id)
        {
            Some(e) => {
                e.source_types.extend(r.source_types);
                e.target_types.extend(r.target_types);
                if e.description.is_empty() {
                    e.description = r.description;
                }
            }
            None => into.relation_types.push(OntologyRelationType { ..r }),
        }
    }
    for t in part.terms {
        let key = normalize_entity_name(&t.name);
        match into
            .terms
            .iter_mut()
            .find(|e| normalize_entity_name(&e.name) == key)
        {
            Some(e) => {
                e.aliases.extend(t.aliases);
                if e.entity_type.is_none() {
                    e.entity_type = t.entity_type;
                }
                if e.description.is_empty() {
                    e.description = t.description;
                }
            }
            None => into.terms.push(t),
        }
    }
}

/// Type ids in a draft so far (to keep later segments consistent).
pub fn known_type_ids(o: &WorkspaceOntology) -> Vec<String> {
    o.entity_types
        .iter()
        .map(|t| normalize_type_token(&t.id))
        .filter(|t| !t.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_with_french_header_quotes_and_aliases() {
        let text = "Terme;Catégorie;Synonymes;Définition\n\"LEAP-1A\";Moteur;\"LEAP 1A|Leap1A\";\"Moteur \"\"CFM\"\" de l'A320neo\"\nPièce X;Pièce;;\n";
        let (o, f) = parse_structured(text, Some("ref.csv")).unwrap().unwrap();
        assert_eq!(f, DraftFormat::Table);
        assert_eq!(o.terms.len(), 2);
        assert_eq!(o.terms[0].aliases, vec!["LEAP 1A", "Leap1A"]);
        assert_eq!(o.terms[0].entity_type.as_deref(), Some("MOTEUR"));
        assert_eq!(o.terms[0].description, "Moteur \"CFM\" de l'A320neo");
        assert_eq!(o.entity_types.len(), 2);
    }

    #[test]
    fn json_object_list_and_plain_text() {
        let (o, _) = parse_structured(r#"{"entity_types":[{"id":"ENGINE"}],"terms":[]}"#, None)
            .unwrap()
            .unwrap();
        assert_eq!(o.entity_types[0].id, "ENGINE");
        let (o, _) = parse_structured(
            r#"["LEAP-1A", {"name":"CFM56","aliases":["CFM-56"]}]"#,
            Some("t.json"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(o.terms.len(), 2);
        assert!(parse_structured(
            "Our engines are described below.\nThe LEAP...",
            Some("ref.md")
        )
        .unwrap()
        .is_none());
        assert!(parse_structured("{not json", Some("x.json")).is_err());
    }

    #[test]
    fn model_answers_with_fences_parse_and_merge() {
        let a = parse_draft_response("```json\n{\"entity_types\":[{\"id\":\"ENGINE\"}],\"terms\":[{\"name\":\"LEAP-1A\",\"aliases\":[\"LEAP 1A\"]}]}\n```").unwrap();
        let b = parse_draft_response("{\"entity_types\":[{\"id\":\"engine\",\"description\":\"A turbine.\"}],\"terms\":[{\"name\":\"Leap-1a\",\"aliases\":[\"L1A\"],\"entity_type\":\"ENGINE\"}]}").unwrap();
        let mut d = WorkspaceOntology::default();
        merge_draft(&mut d, a);
        merge_draft(&mut d, b);
        assert_eq!(d.entity_types.len(), 1);
        assert_eq!(d.entity_types[0].description, "A turbine.");
        assert_eq!(d.terms.len(), 1);
        assert_eq!(d.terms[0].aliases, vec!["LEAP 1A", "L1A"]);
        assert_eq!(d.terms[0].entity_type.as_deref(), Some("ENGINE"));
        assert!(parse_draft_response("no json").is_err());
    }

    #[test]
    fn segments_respect_paragraphs_and_cap() {
        let text = format!("{}\n{}\n", "a".repeat(8_000), "b".repeat(8_000));
        let (s, truncated) = segments(&text);
        assert_eq!(s.len(), 2);
        assert!(!truncated);
        let (s, truncated) = segments(&"x".repeat(MAX_DRAFT_TEXT_CHARS + 10));
        assert!(truncated);
        assert_eq!(
            s.iter().map(|p| p.chars().count()).sum::<usize>(),
            MAX_DRAFT_TEXT_CHARS
        );
    }
}
