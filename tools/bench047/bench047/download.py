"""Download MMLongBench-Doc (CC BY-NC 4.0 — research use only)."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Iterable, Optional

from huggingface_hub import hf_hub_download, list_repo_files

from .paths import dataset_root, documents_dir, qa_parquet_path

REPO_ID = "yubo2333/MMLongBench-Doc"
NC_NOTICE = (
    "MMLongBench-Doc data is licensed CC BY-NC 4.0 (research / non-commercial only). "
    "Do not redistribute PDFs. Cite arXiv:2407.01523."
)


def download_qa() -> Path:
    print(NC_NOTICE)
    root = dataset_root()
    root.mkdir(parents=True, exist_ok=True)
    path = hf_hub_download(
        repo_id=REPO_ID,
        repo_type="dataset",
        filename="data/train-00000-of-00001.parquet",
        local_dir=str(root),
    )
    print(f"QA parquet: {path}")
    return Path(path)


def list_pdf_files() -> list[str]:
    files = list_repo_files(REPO_ID, repo_type="dataset")
    return sorted(f for f in files if f.startswith("documents/") and f.endswith(".pdf"))


def download_pdfs(doc_ids: Optional[Iterable[str]] = None) -> list[Path]:
    """Download PDFs. If doc_ids given, only those filenames; else all."""
    print(NC_NOTICE)
    docs = documents_dir()
    docs.mkdir(parents=True, exist_ok=True)
    wanted = set(doc_ids) if doc_ids is not None else None
    out: list[Path] = []
    for rel in list_pdf_files():
        name = Path(rel).name
        if wanted is not None and name not in wanted:
            continue
        path = hf_hub_download(
            repo_id=REPO_ID,
            repo_type="dataset",
            filename=rel,
            local_dir=str(dataset_root()),
        )
        # huggingface may place under documents/
        p = Path(path)
        if not p.exists():
            p = docs / name
        out.append(p)
        print(f"  pdf: {name}")
    _write_manifest(out)
    return out


def _write_manifest(paths: list[Path]) -> None:
    rows = []
    for p in paths:
        h = hashlib.sha256(p.read_bytes()).hexdigest()
        rows.append({"path": str(p), "name": p.name, "sha256": h, "bytes": p.stat().st_size})
    man = dataset_root() / "manifest.json"
    man.write_text(json.dumps({"repo": REPO_ID, "files": rows}, indent=2))
    print(f"Manifest: {man} ({len(rows)} files)")


def ensure_qa() -> Path:
    p = qa_parquet_path()
    if not p.exists():
        return download_qa()
    return p
