'use client';

/**
 * @module RelationTypeSelector
 * @description Relation type allow-list picker (SPEC-114). Twin of EntityTypeSelector
 * without domain presets or color swatches.
 */

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  MAX_RELATION_TYPES,
  normalizeRelationType,
} from '@/constants/kg-schema-presets';
import { deduplicateTypes } from '@/constants/entity-presets';
import { cn } from '@/lib/utils';
import { Plus, X } from 'lucide-react';
import { KeyboardEvent, useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';

export interface RelationTypeSelectorProps {
  value: string[];
  onChange: (types: string[]) => void;
  readOnly?: boolean;
  strictLimit?: boolean;
  onStrictLimitChange?: (strict: boolean) => void;
  /** Optional className for the chip scroll area (SPEC-114 wide layout). */
  chipAreaClassName?: string;
  /** Compact spacing for dense wizard columns (SPEC-114 layout). */
  density?: 'default' | 'compact';
}

export function RelationTypeSelector({
  value,
  onChange,
  readOnly = false,
  strictLimit = true,
  onStrictLimitChange,
  chipAreaClassName,
  density = 'default',
}: RelationTypeSelectorProps) {
  const { t } = useTranslation();
  const [customInput, setCustomInput] = useState('');
  const [bulkInput, setBulkInput] = useState('');
  const atMax = value.length >= MAX_RELATION_TYPES;
  const isCompact = density === 'compact';
  const stackGap = isCompact ? 'space-y-2' : 'space-y-3';

  const handleRemove = useCallback(
    (type: string) => {
      if (readOnly) return;
      onChange(value.filter((t) => t !== type));
    },
    [onChange, readOnly, value],
  );

  const handleAdd = useCallback(() => {
    if (readOnly) return;
    const normalized = normalizeRelationType(customInput);
    if (!normalized) return;
    onChange(deduplicateTypes([...value, normalized]));
    setCustomInput('');
  }, [customInput, onChange, readOnly, value]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handleAdd();
      }
    },
    [handleAdd],
  );

  const handleApplyBulk = useCallback(() => {
    if (readOnly) return;
    const parsed = bulkInput
      .split(/[\n,;]+/)
      .map((entry) => normalizeRelationType(entry))
      .filter(Boolean);
    onChange(deduplicateTypes(parsed));
  }, [bulkInput, onChange, readOnly]);

  return (
    <div
      className={stackGap}
      data-testid="relation-type-selector"
      aria-label={t('relationTypes.selectorLabel', 'Relation type selector')}
    >
      <Tabs defaultValue="basic" className={stackGap}>
        <TabsList className="grid w-full grid-cols-2 h-8">
          <TabsTrigger value="basic" data-testid="relation-tab-basic" className="text-xs">
            {t('relationTypes.tabList', 'List')}
          </TabsTrigger>
          <TabsTrigger value="advanced" data-testid="relation-tab-advanced" className="text-xs">
            {t('relationTypes.tabBulkEdit', 'Bulk edit')}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="basic" className={`${stackGap} m-0`}>
          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
              {!isCompact ? (
                <p className="text-xs font-medium text-muted-foreground">
                  {t('relationTypes.typesLabel', 'Relation types')}
                </p>
              ) : (
                <span className="sr-only">
                  {t('relationTypes.typesLabel', 'Relation types')}
                </span>
              )}
              <span
                className={`text-xs ${atMax ? 'text-destructive' : 'text-muted-foreground'} ml-auto`}
                aria-live="polite"
              >
                {value.length}/{MAX_RELATION_TYPES}
              </span>
            </div>
            <div
              className={cn(
                'min-h-10 max-h-24 overflow-y-auto flex flex-wrap gap-1.5 p-2 rounded-md border bg-background',
                chipAreaClassName,
              )}
              data-testid="relation-types-chips"
            >
              {value.length === 0 && (
                <span className="text-xs text-muted-foreground italic self-center">
                  {t(
                    'relationTypes.emptyHint',
                    'No relation types — model may use free-form labels',
                  )}
                </span>
              )}
              {value.map((type) => (
                <Badge
                  key={type}
                  variant="secondary"
                  className="gap-1.5 pr-1 text-xs font-mono"
                  data-testid={`relation-type-chip-${type}`}
                >
                  {type}
                  {!readOnly && (
                    <button
                      type="button"
                      className="ml-0.5 rounded-sm hover:bg-muted-foreground/20 p-0.5 transition-colors"
                      onClick={() => handleRemove(type)}
                      aria-label={t('relationTypes.removeType', 'Remove {{type}}', {
                        type,
                      })}
                      data-testid={`remove-relation-${type}`}
                    >
                      <X className="h-3 w-3" />
                    </button>
                  )}
                </Badge>
              ))}
            </div>
          </div>

          {onStrictLimitChange && value.length > 0 && (
            <div className="flex items-start gap-2 rounded-md border bg-muted/30 p-2">
              <Checkbox
                id="relation-types-strict-limit"
                checked={strictLimit}
                onCheckedChange={(checked) => onStrictLimitChange(checked === true)}
                disabled={readOnly}
                data-testid="relation-types-strict-checkbox"
                className="mt-0.5"
              />
              <div className="space-y-0.5 min-w-0">
                <Label
                  htmlFor="relation-types-strict-limit"
                  className="text-xs font-medium leading-snug cursor-pointer"
                >
                  {t(
                    'relationTypes.strictLimitLabel',
                    'Limit extraction to listed relations (remap others)',
                  )}
                </Label>
                {!isCompact ? (
                  <p className="text-xs text-muted-foreground">
                    {t(
                      'relationTypes.strictLimitHint',
                      'When off, the model may use additional relation labels.',
                    )}
                  </p>
                ) : null}
              </div>
            </div>
          )}

          {!readOnly && (
            <div className="flex gap-2">
              <Input
                placeholder={t('relationTypes.addPlaceholder', 'e.g. WORKS_AT')}
                value={customInput}
                onChange={(e) => setCustomInput(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={atMax}
                className="h-8 text-xs font-mono uppercase"
                data-testid="relation-type-input"
              />
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8 px-3 shrink-0"
                onClick={handleAdd}
                disabled={atMax || !customInput.trim()}
                data-testid="relation-type-add-btn"
              >
                <Plus className="h-3.5 w-3.5 mr-1" />
                {t('common.add', 'Add')}
              </Button>
            </div>
          )}
        </TabsContent>

        <TabsContent value="advanced" className="m-0 space-y-2">
          <textarea
            value={bulkInput}
            onChange={(e) => setBulkInput(e.target.value)}
            placeholder={t(
              'relationTypes.bulkPlaceholder',
              'WORKS_AT, PART_OF or one per line',
            )}
            disabled={readOnly}
            className="w-full min-h-20 rounded-md border bg-background px-3 py-2 text-xs font-mono"
            data-testid="relation-advanced-bulk-input"
          />
          <div className="flex gap-2">
            <Button
              type="button"
              size="sm"
              onClick={handleApplyBulk}
              disabled={readOnly || !bulkInput.trim()}
              data-testid="relation-advanced-apply-bulk"
            >
              {t('relationTypes.applyBulk', 'Apply bulk list')}
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={() => onChange([])}
              disabled={readOnly || value.length === 0}
              data-testid="relation-advanced-clear-all"
            >
              {t('relationTypes.clearAll', 'Clear all')}
            </Button>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default RelationTypeSelector;
