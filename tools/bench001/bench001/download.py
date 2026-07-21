"""Download GraphRAG-Bench corpus + questions from Hugging Face."""

from __future__ import annotations

from pathlib import Path

from huggingface_hub import hf_hub_download, snapshot_download

from .paths import DATASET_ID, DATASET_REVISION, dataset_root


FILES = (
    "Datasets/Questions/medical_questions.json",
    "Datasets/Questions/novel_questions.json",
    "Datasets/Corpus/medical.json",
    "Datasets/Corpus/novel.json",
)


def download_dataset(*, revision: str = DATASET_REVISION) -> Path:
    """Download required GraphRAG-Bench files into local cache."""
    root = dataset_root()
    root.mkdir(parents=True, exist_ok=True)
    # Prefer per-file download for speed; fall back to snapshot.
    try:
        for rel in FILES:
            local = hf_hub_download(
                repo_id=DATASET_ID,
                filename=rel,
                repo_type="dataset",
                revision=revision,
                local_dir=str(root),
            )
            print(f"downloaded {rel} -> {local}")
    except Exception as exc:  # noqa: BLE001
        print(f"per-file download failed ({exc}); using snapshot_download")
        snapshot_download(
            repo_id=DATASET_ID,
            repo_type="dataset",
            revision=revision,
            local_dir=str(root),
            allow_patterns=["Datasets/Questions/*", "Datasets/Corpus/*"],
        )
    return root
