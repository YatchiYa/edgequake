<?php

declare(strict_types=1);

namespace EdgeQuake;

/**
 * HTTP 202 admit payload for DELETE /api/v1/documents (workspace wipe).
 */
final class DeleteAllResponse
{
    public function __construct(
        public readonly bool $accepted = false,
        public readonly ?string $wipeTrackId = null,
        public readonly int $deletedCount = 0,
        public readonly ?int $plannedDeleteCount = null,
        public readonly int $totalChunksDeleted = 0,
        public readonly int $totalEntitiesRemoved = 0,
        public readonly int $totalRelationshipsRemoved = 0,
        public readonly int $totalPdfsDeleted = 0,
        public readonly int $skippedCount = 0,
        /** @var list<string> */
        public readonly array $skippedDocuments = [],
        public readonly ?string $message = null,
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        $skipped = $data['skipped_documents'] ?? [];
        if (!is_array($skipped)) {
            $skipped = [];
        }

        return new self(
            accepted: (bool) ($data['accepted'] ?? false),
            wipeTrackId: isset($data['wipe_track_id']) ? (string) $data['wipe_track_id'] : null,
            deletedCount: (int) ($data['deleted_count'] ?? 0),
            plannedDeleteCount: isset($data['planned_delete_count'])
                ? (int) $data['planned_delete_count']
                : null,
            totalChunksDeleted: (int) ($data['total_chunks_deleted'] ?? 0),
            totalEntitiesRemoved: (int) ($data['total_entities_removed'] ?? 0),
            totalRelationshipsRemoved: (int) ($data['total_relationships_removed'] ?? 0),
            totalPdfsDeleted: (int) ($data['total_pdfs_deleted'] ?? 0),
            skippedCount: (int) ($data['skipped_count'] ?? 0),
            skippedDocuments: array_values(array_map('strval', $skipped)),
            message: isset($data['message']) ? (string) $data['message'] : null,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        return [
            'accepted' => $this->accepted,
            'wipe_track_id' => $this->wipeTrackId,
            'deleted_count' => $this->deletedCount,
            'planned_delete_count' => $this->plannedDeleteCount,
            'total_chunks_deleted' => $this->totalChunksDeleted,
            'total_entities_removed' => $this->totalEntitiesRemoved,
            'total_relationships_removed' => $this->totalRelationshipsRemoved,
            'total_pdfs_deleted' => $this->totalPdfsDeleted,
            'skipped_count' => $this->skippedCount,
            'skipped_documents' => $this->skippedDocuments,
            'message' => $this->message,
        ];
    }
}
