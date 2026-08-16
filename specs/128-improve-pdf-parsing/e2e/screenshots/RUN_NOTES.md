# SPEC-128 overlay run notes

### S01
- Overlay off; real PDFViewer chrome; 0 pdf-layout-box
### S02
- Figures chip default on; CSS IoU vs bbox_norm=1.000
### S03
- Paragraphs chip reveals paragraph box from GET layout
### S04
- Zoom 150%; IoU=1.000
### S05
- Noise chip shows abandon (not RAG-indexed)

## M live 01-the-abondance-inversion.pdf mistral-small-latest
- document_id: `01a0097a-9cc8-71ba-a993-5831fbc7fd9c`
- vision=mistral/mistral-small-latest; classes=figure,paragraph,column
- Unmocked GET layout + ingested PDF bytes

- live CSS IoU vs GET bbox_norm (figure)=1.000

## M corpus (remaining pdf_data)
- Sequential mistral-small-latest admit + layout poll (no KG wait)
- coin_rag2608.07458v1.pdf, kg_2608.09779v1.pdf, ssm2608.02560v1.pdf each ≥1 persisted region

