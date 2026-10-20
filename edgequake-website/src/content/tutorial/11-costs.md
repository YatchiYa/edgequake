---
title: "Costs"
description: "EdgeQuake picture tour — Costs"
order: 11
---

[Contents](/tutorial/) · Previous: [Pipeline](/tutorial/10-pipeline/) · Next: [API Explorer](/tutorial/12-api-explorer/)

# Costs

Costs is the bill for model calls in this workspace. Reading a file and answering a question both call a model, and both show up here.

![The Cost Dashboard. Total cost is $1.47 across 12 documents. Extraction is most of the bar. A token table sits at the bottom.](/tutorial/11-costs.png)

The title is **Cost Dashboard**. The subtitle on the screen is **Monitor LLM costs and usage across your workspace**. LLM is the language model: the program that reads files and writes answers.

**Last 30 days** is the range for the trend chart. The refresh icon reloads the numbers. **Download** saves the table.

## The summary

**Cost Summary**, labeled **All time**, has four figures for this demo:

- **Total Cost $1.47**
- **Documents 12**
- **Avg per Document $0.122**
- **Tokens Used 3.7M**

A token is a small piece of text the model is billed on. A short sentence is a handful of tokens. 3.7 million is the sum of everything sent in and everything the model wrote back.

**Budget Status** is a monthly cap. It shows **$0.00 of $100.00**, **0% used**, and **$100.00 remaining**. The **Monthly** pill and **Resets in** tell you the cap is per month. The $1.47 in the summary is all-time spend, so it can be higher than the month-to-date number in the budget card. On this day the current month had not added new spend.

## Where the money went

**Cost Breakdown**, total **$1.47**:

- **Extraction $1.32**, the blue bar, about **90%**. This is the work of reading files and listing entities.
- **Embedding $0.147**, the purple bar, about **10%**. This is the work of turning chunks into vectors.

**Cost Trend** is a chart of spend across days. The labels along the bottom are dates. A flat chart means little new work in that window.

## The token table

**Token Usage Details** says **3.7M total tokens**. The columns are **Stage**, **Input**, **Output**, **Calls**, and **Cost**.

| Stage | Input | Output | Calls | Cost |
| --- | --- | --- | --- | --- |
| Extraction | 1.7M | 2.0M | 12 | $1.32 |
| Embedding | 0 | 0 | 12 | $0.147 |
| Total | 1.7M | 2.0M | 24 | $1.47 |

Input is text sent to the model. Output is text the model returned. Embedding shows zeros in the token columns and still shows a cost, because that provider bills the vector step differently. **Calls** is how many times that stage ran: 12, once per document, and 24 in total.

These figures belong to the public demo. Your own workspace will show your own bill.

## Keep going

Next: [API Explorer](/tutorial/12-api-explorer/). The queue that generates most of this bill is [Pipeline](/tutorial/10-pipeline/).
