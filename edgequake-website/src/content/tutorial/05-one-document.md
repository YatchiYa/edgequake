---
title: "One document"
description: "EdgeQuake picture tour — One document"
order: 5
---

[Contents](/tutorial/) · Previous: [Documents](/tutorial/04-documents/) · Next: [Ask a question](/tutorial/06-ask-a-question/)

# One document

A document has two views. A short preview sits beside the list. The full page shows the extracted text, the counts, and the steps that produced them.

## The preview

Click a row once on [Documents](/tutorial/04-documents/). A panel named **Document Preview** opens on the right. You can close it with the **×**.

![The library with the preview open for bricks_2610.12452v1.pdf. The panel shows Completed, 1,965 entities, a cost of $0.16, and the start of the extracted text.](/tutorial/05-document-preview.png)

For `bricks_2610.12452v1.pdf` the panel shows:

- A green **Completed** pill, **1,965 entities**, and **2 days ago**.
- **Details**: an **ID** you can copy, plus **Created** and **Updated**.
- **Processing cost**: **Total Cost $0.16**, **Total Tokens 430.1K**, split into **Input Tokens 218.2K** and **Output Tokens 211.8K**, the **LLM Model** `mistral-small-latest`, and the **Embedding** model `mistral-embed`.
- **Content preview**, with a **Copy** button and **Show More** when the text is long.

This is a glance. The full page is where you read the file.

## The full page

Double-click the row, or open the file from the preview. The address changes to a page for that one document.

![The open PDF bricks_2610.12452v1.pdf. Extracted markdown is in the middle, with a picture of robots. The Details column on the right lists chunks, entities, relations, and the extraction steps.](/tutorial/05-document.png)

The breadcrumb at the top reads **EdgeQuake / Documents / bricks_2610.12452v1.pdf**. Under that is the file name again.

The toolbar on the right has:

- A download icon, which saves the file.
- **Ask**, which opens [Query](/tutorial/06-ask-a-question/) with this file in mind.
- **Reprocess specific pages**, which sends chosen pages through reading again.
- **Reprocess**, which sends the whole file through again.

Use those reprocess buttons only on a collection you own. They spend model calls and rewrite the map for this file.

### The reader

The middle is the text the software extracted. This file has **22** pages. The controls above the text are:

- **‹ 1 / 22 ›** moves between pages.
- The magnifying glasses and **100%** zoom in and out.
- **Layout** changes the arrangement.
- **Extracted Markdown** is the text you are looking at. Markdown is the simple text format the reader uses, with headings and pictures.
- **None**, **PDF → MD**, and **MD → PDF** choose what you compare. **PDF → MD** is the useful one: the original page on one side and the extracted text on the other. **MD → PDF** flips that comparison. **None** shows a single pane.
- The copy icons copy the extracted text.

Page 1 of this paper is a picture of brick robots. The extracted text starts with **Subject** and **Key visual elements labelled**, then a one-sentence caption. That is the model describing the picture, because the page is an image, not a paragraph of typed words.

### The details column

**Details** on the right has four tiles:

- **Chunks 59** — how many pieces this file was cut into.
- **Entities 1965** — how many things were found.
- **Relations 1497** — how many lines were found.
- **Processed 1.4m** — how long that work took.

**Extraction Lineage** is the path from the file to the map. Each step is a row:

1. **Document Upload** — the file arrived.
2. **PDF → Markdown (Vision)** — a vision model, here `magistral-small-latest`, read the pages.
3. **Content Extraction** — the text was stored, here about 3160 characters per chunk.
4. **Entity Extraction** — a chat model, here `mistral-small-latest`, listed entities. **12 types** is how many kinds of thing it was allowed to use (person, organization, and so on).
5. **Relationship Mapping** — the lines between those entities were stored.

If a step had failed, this column is where you would see it. This file is green all the way down.

## Keep going

Next: [Ask a question](/tutorial/06-ask-a-question/). To see this file's entities as dots, open [the map](/tutorial/07-the-map/) and search for a name you saw in the text.
