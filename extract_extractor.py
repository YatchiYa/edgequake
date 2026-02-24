#!/usr/bin/env python3
"""Extract extractor.rs into sub-modules."""
import os
import shutil

SRC = "edgequake/crates/edgequake-pipeline/src/extractor.rs"
DST = "edgequake/crates/edgequake-pipeline/src/extractor"

with open(SRC, "r") as f:
    lines = f.readlines()

total = len(lines)
print(f"Total lines: {total}")

# Create output directory
os.makedirs(DST, exist_ok=True)

# Line ranges (1-indexed, inclusive):
# Types+trait+helper+tests = 1-300, 300-335, 580-596, 1430-1514
# SimpleExtractor = 336-395
# LLMExtractor = 397-579
# SOTAExtractor = 597-1100
# GleaningConfig+GleaningExtractor = 1101-1429
# Tests = 1430-1514


# Precise identification of boundaries by searching for markers
def find_line(pattern, start=0):
    for i in range(start, total):
        if pattern in lines[i]:
            return i
    return -1


def find_block_end(start_line):
    """Find the end of a brace-delimited block starting at start_line.
    Skips braces inside string literals and character literals."""
    brace_depth = 0
    found_open = False
    for i in range(start_line, total):
        line = lines[i]
        j = 0
        while j < len(line):
            ch = line[j]
            # Skip string literals
            if ch == '"':
                j += 1
                while j < len(line) and line[j] != '"':
                    if line[j] == "\\":
                        j += 1  # skip escaped char
                    j += 1
                j += 1  # skip closing quote
                continue
            # Skip character literals
            if ch == "'" and j + 2 < len(line):
                # Check for 'x' or '\x' patterns
                if line[j + 1] == "\\" and j + 3 < len(line) and line[j + 3] == "'":
                    j += 4
                    continue
                elif j + 2 < len(line) and line[j + 2] == "'":
                    j += 3
                    continue
            if ch == "{":
                brace_depth += 1
                found_open = True
            elif ch == "}":
                brace_depth -= 1
                if brace_depth == 0 and found_open:
                    return i + 1
            j += 1
    return total


# Find boundaries
simple_struct = find_line("pub struct SimpleExtractor")
llm_doc_start = find_line("/// LLM-based entity extractor")
extract_json_fn = find_line("fn extract_json_from_response")
sota_doc_start = find_line("/// SOTA LLM-based entity extractor")
gleaning_config = find_line("pub struct GleaningConfig")
# Find the doc comment start for GleaningConfig
gleaning_config_doc = gleaning_config
while gleaning_config_doc > 0 and (
    lines[gleaning_config_doc - 1].strip().startswith("///")
    or lines[gleaning_config_doc - 1].strip().startswith("#[derive")
):
    gleaning_config_doc -= 1

gleaning_extractor_doc = find_line("/// A wrapper extractor that performs gleaning")
tests_start = find_line("#[cfg(test)]")

print(f"SimpleExtractor struct: line {simple_struct+1}")
print(f"LLMExtractor doc: line {llm_doc_start+1}")
print(f"extract_json_from_response fn: line {extract_json_fn+1}")
print(f"SOTAExtractor doc: line {sota_doc_start+1}")
print(f"GleaningConfig doc: line {gleaning_config_doc+1}")
print(f"GleaningConfig struct: line {gleaning_config+1}")
print(f"GleaningExtractor doc: line {gleaning_extractor_doc+1}")
print(f"Tests: line {tests_start+1}")

# Now find the start of SimpleExtractor's doc comment
simple_doc = simple_struct
while simple_doc > 0 and (
    lines[simple_doc - 1].strip().startswith("///")
    or lines[simple_doc - 1].strip() == ""
):
    simple_doc -= 1
# Back up one more if we went too far
if simple_doc < simple_struct and not lines[simple_doc].strip().startswith("///"):
    simple_doc += 1

print(f"SimpleExtractor doc start: line {simple_doc+1}")

# Find where SimpleExtractor ends (after EntityExtractor impl block)
simple_impl_start = find_line("impl EntityExtractor for SimpleExtractor", simple_struct)
simple_end = find_block_end(simple_impl_start)

print(f"SimpleExtractor end: line {simple_end}")

# Find where LLMExtractor ends (after EntityExtractor impl block)
llm_impl_start = find_line("impl<L> EntityExtractor for LLMExtractor<L>", llm_doc_start)
llm_end = find_block_end(llm_impl_start)

print(f"LLMExtractor end: line {llm_end}")

# Find where extract_json_from_response ends
# Hardcoded: the function is short (~20 lines), ends before SOTAExtractor doc comment
# Can't use brace counting due to r#"..."# raw strings in nearby format! macros
extract_json_end = sota_doc_start  # Next section starts right after
# Walk backwards to skip empty lines between function end and next doc comment
while extract_json_end > extract_json_fn and lines[extract_json_end - 1].strip() == "":
    extract_json_end -= 1

print(f"extract_json_from_response end: line {extract_json_end}")

# Find where SOTAExtractor ends
sota_impl_start = find_line(
    "impl<L> EntityExtractor for SOTAExtractor<L>", sota_doc_start
)
sota_end = find_block_end(sota_impl_start)

print(f"SOTAExtractor end: line {sota_end}")

# ---- Write mod.rs ----
# Types + trait (lines 0 to simple_doc-1), extract_json_from_response, tests
with open(os.path.join(DST, "mod.rs"), "w") as f:
    # Original module doc + imports + types + trait (up to SimpleExtractor)
    for line in lines[0:simple_doc]:
        f.write(line)

    # extract_json_from_response helper (used by multiple extractors)
    f.write("\n")
    for line in lines[extract_json_fn:extract_json_end]:
        f.write(line)

    # Sub-module declarations and re-exports
    f.write("\n")
    f.write("mod simple;\n")
    f.write("mod llm;\n")
    f.write("mod sota;\n")
    f.write("mod gleaning;\n")
    f.write("\n")
    f.write("pub use simple::SimpleExtractor;\n")
    f.write("pub use llm::LLMExtractor;\n")
    f.write("pub use sota::SOTAExtractor;\n")
    f.write("pub use gleaning::{GleaningConfig, GleaningExtractor};\n")

    # Tests
    f.write("\n")
    for line in lines[tests_start:]:
        f.write(line)

print(f"mod.rs written")

# ---- Write simple.rs ----
with open(os.path.join(DST, "simple.rs"), "w") as f:
    f.write("//! Simple regex-based entity extractor for testing.\n\n")
    f.write("use async_trait::async_trait;\n")
    f.write("use std::collections::HashMap;\n\n")
    f.write("use crate::chunker::TextChunk;\n")
    f.write("use crate::error::{PipelineError, Result};\n")
    f.write("use super::{EntityExtractor, ExtractedEntity, ExtractionResult};\n\n")

    for line in lines[simple_doc:simple_end]:
        f.write(line)

print(f"simple.rs written")

# ---- Write llm.rs ----
with open(os.path.join(DST, "llm.rs"), "w") as f:
    f.write("//! LLM-based entity extractor using structured JSON prompts.\n\n")
    f.write("use async_trait::async_trait;\n")
    f.write("use edgequake_llm::traits::ChatMessage;\n\n")
    f.write("use crate::chunker::TextChunk;\n")
    f.write("use crate::error::{PipelineError, Result};\n")
    f.write(
        "use super::{EntityExtractor, ExtractedEntity, ExtractedRelationship, ExtractionResult, extract_json_from_response};\n\n"
    )

    for line in lines[llm_doc_start:llm_end]:
        f.write(line)

print(f"llm.rs written")

# ---- Write sota.rs ----
with open(os.path.join(DST, "sota.rs"), "w") as f:
    f.write("//! SOTA LLM-based entity extractor using tuple-format prompts.\n")
    f.write("//!\n")
    f.write("//! @implements FEAT0303\n\n")
    f.write("use async_trait::async_trait;\n")
    f.write("use edgequake_llm::traits::{ChatMessage, CompletionOptions};\n\n")
    f.write("use crate::chunker::TextChunk;\n")
    f.write("use crate::error::{PipelineError, Result};\n")
    f.write("use super::{EntityExtractor, ExtractionResult};\n\n")

    for line in lines[sota_doc_start:sota_end]:
        f.write(line)

print(f"sota.rs written")

# ---- Write gleaning.rs ----
with open(os.path.join(DST, "gleaning.rs"), "w") as f:
    f.write("//! Gleaning (re-extraction) extractor for finding missed entities.\n")
    f.write("//!\n")
    f.write("//! @implements FEAT0305\n\n")
    f.write("use async_trait::async_trait;\n")
    f.write("use serde::{Deserialize, Serialize};\n\n")
    f.write("use crate::chunker::TextChunk;\n")
    f.write("use crate::error::{PipelineError, Result};\n")
    f.write(
        "use super::{EntityExtractor, ExtractedEntity, ExtractedRelationship, ExtractionResult, extract_json_from_response};\n\n"
    )

    for line in lines[gleaning_config_doc:tests_start]:
        f.write(line)

print(f"gleaning.rs written")

# Count lines
for fname in ["mod.rs", "simple.rs", "llm.rs", "sota.rs", "gleaning.rs"]:
    path = os.path.join(DST, fname)
    with open(path) as f:
        count = sum(1 for _ in f)
    print(f"  {fname}: {count} lines")

# Backup original
shutil.copy2(SRC, SRC.replace(".rs", "_old.rs"))
print(f"\nBackup created: {SRC.replace('.rs', '_old.rs')}")
print("Now delete the original and test compilation.")
