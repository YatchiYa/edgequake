/**
 * @module WorkspaceEntityTypesCard
 * @description Entity type list + strict-limit editor for workspace settings (dashboard and deeplink).
 *
 * @implements SPEC-085 / GitHub #216 — editable entity_types
 * @implements SPEC-013 entity_extraction — entity_types_strict toggle
 */
'use client';

import { EntityTypeSelector } from '@/components/shared/entity-type-selector';
import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import type { Workspace } from '@/types';
import { Tags } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export interface WorkspaceEntityTypesCardProps {
  isEditing: boolean;
  workspace: Workspace;
  selectedTypes: string[];
  onTypesChange: (types: string[]) => void;
  strictLimit: boolean;
  onStrictLimitChange: (strict: boolean) => void;
  /** SPEC-096 LAW-L6 — drives localized preset tokens in the editor. */
  extractionLanguage?: string | null;
}

export function WorkspaceEntityTypesCard({
  isEditing,
  workspace,
  selectedTypes,
  onTypesChange,
  strictLimit,
  onStrictLimitChange,
  extractionLanguage = null,
}: WorkspaceEntityTypesCardProps) {
  const { t } = useTranslation();

  return (
    <Card className="gap-2 py-4 h-full" data-testid="workspace-entity-types-card">
      <CardHeader className="px-4 pb-0 gap-1">
        <CardTitle className="flex items-center gap-2 text-base">
          <Tags className="h-4 w-4 text-indigo-600" />
          {t('entityTypes.title', 'Entity Types')}
        </CardTitle>
        <CardDescription className="text-xs leading-snug">
          {t(
            'entityTypes.futureOnlyHint',
            'Applies to future document ingestions. Use Rebuild Knowledge Graph to re-extract existing documents.'
          )}
        </CardDescription>
      </CardHeader>
      <CardContent className="px-4">
        {isEditing ? (
          <EntityTypeSelector
            value={selectedTypes}
            onChange={onTypesChange}
            strictLimit={strictLimit}
            onStrictLimitChange={onStrictLimitChange}
            extractionLanguage={extractionLanguage}
          />
        ) : workspace.entity_types && workspace.entity_types.length > 0 ? (
          <div className="space-y-2">
            <div className="flex flex-wrap gap-1.5">
              {workspace.entity_types.map((type) => (
                <Badge
                  key={type}
                  variant="secondary"
                  className="text-xs font-mono"
                  data-testid={`ws-entity-type-${type}`}
                >
                  {type}
                </Badge>
              ))}
            </div>
            <p className="text-xs text-muted-foreground" data-testid="entity-types-strict-status">
              {workspace.entity_types_strict !== false
                ? t('entityTypes.strictOn', 'Strict limit: on (unknown types → OTHER)')
                : t('entityTypes.strictOff', 'Strict limit: off (free-form types allowed)')}
            </p>
          </div>
        ) : (
          <div className="space-y-1.5 rounded-md border border-dashed bg-muted/20 px-2.5 py-2 text-xs text-muted-foreground">
            <span className="font-medium text-foreground/80">
              {t('entityTypes.defaults', 'Using server defaults:')}
            </span>{' '}
            <span className="font-mono">
              {t(
                'entityTypes.defaultsHint',
                'PERSON, CREATURE, ORGANIZATION, LOCATION, EVENT, CONCEPT, METHOD, CONTENT, DATA, ARTIFACT, NATURALOBJECT, OTHER'
              )}
            </span>
            <p data-testid="entity-types-strict-status">
              {workspace.entity_types_strict !== false
                ? t('entityTypes.strictOn', 'Strict limit: on (unknown types → OTHER)')
                : t('entityTypes.strictOff', 'Strict limit: off (free-form types allowed)')}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
