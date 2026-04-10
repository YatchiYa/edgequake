'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { getWorkspace, updateWorkspace } from '@/lib/api/edgequake';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Eye, Gauge, Pencil, Save, X } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

type BackendChoice = 'none' | 'vision' | 'edgeparse';

function backendLabel(value: BackendChoice) {
  switch (value) {
    case 'edgeparse':
      return 'EdgeParse';
    case 'vision':
      return 'Vision';
    default:
      return 'Server Default';
  }
}

export function PdfParserSettingsCard() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { selectedTenantId, selectedWorkspaceId } = useTenantStore();
  const [isEditing, setIsEditing] = useState(false);
  const [backend, setBackend] = useState<BackendChoice>('none');

  const { data: workspace, isLoading } = useQuery({
    queryKey: ['workspace', selectedTenantId, selectedWorkspaceId],
    queryFn: () => getWorkspace(selectedTenantId!, selectedWorkspaceId!),
    enabled: !!selectedTenantId && !!selectedWorkspaceId,
    staleTime: 60000,
    retry: 1,
  });

  useEffect(() => {
    if (!workspace || isEditing) {
      return;
    }
    setBackend((workspace.pdf_parser_backend as BackendChoice | undefined) ?? 'none');
  }, [workspace, isEditing]);

  const updateMutation = useMutation({
    mutationFn: () =>
      updateWorkspace(selectedTenantId!, selectedWorkspaceId!, {
        pdf_parser_backend: backend === 'none' ? 'none' : backend,
      }),
    onSuccess: () => {
      toast.success(
        t('settings.pdfParser.updateSuccess', 'PDF parser default updated'),
      );
      queryClient.invalidateQueries({
        queryKey: ['workspace', selectedTenantId, selectedWorkspaceId],
      });
      setIsEditing(false);
    },
    onError: (error) => {
      toast.error(
        t('settings.pdfParser.updateFailed', 'Failed to update PDF parser default'),
        {
          description: error instanceof Error ? error.message : 'Unknown error',
        },
      );
    },
  });

  if (!selectedTenantId || !selectedWorkspaceId) {
    return null;
  }

  return (
    <Card>
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Gauge className="h-5 w-5 text-amber-600" />
            <CardTitle>{t('settings.pdfParser.title', 'PDF Parser')}</CardTitle>
          </div>
          {!isEditing && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsEditing(true)}
              aria-label={t('common.edit', 'Edit')}
            >
              <Pencil className="h-4 w-4" />
            </Button>
          )}
        </div>
        <CardDescription>
          {t(
            'settings.pdfParser.subtitle',
            'Choose the default PDF extraction backend for this workspace. EdgeParse is faster and deterministic; Vision is better for scanned or image-heavy PDFs.',
          )}
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-4">
        {isLoading ? (
          <Skeleton className="h-14 w-full" />
        ) : isEditing ? (
          <>
            <Select
              value={backend}
              onValueChange={(value: BackendChoice) => setBackend(value)}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">
                  {t('settings.pdfParser.serverDefault', 'Server Default')}
                </SelectItem>
                <SelectItem value="vision">
                  {t('settings.pdfParser.vision', 'Vision')}
                </SelectItem>
                <SelectItem value="edgeparse">
                  {t('settings.pdfParser.edgeparse', 'EdgeParse')}
                </SelectItem>
              </SelectContent>
            </Select>
            <div className="flex items-center gap-2 pt-2">
              <Button
                size="sm"
                onClick={() => updateMutation.mutate()}
                disabled={updateMutation.isPending}
              >
                <Save className="h-4 w-4 mr-2" />
                {t('common.save', 'Save')}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  setBackend(
                    (workspace?.pdf_parser_backend as BackendChoice | undefined) ??
                      'none',
                  );
                  setIsEditing(false);
                }}
                disabled={updateMutation.isPending}
              >
                <X className="h-4 w-4 mr-2" />
                {t('common.cancel', 'Cancel')}
              </Button>
            </div>
          </>
        ) : workspace ? (
          <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
            {backend === 'vision' ? (
              <Eye className="h-4 w-4 text-orange-600" />
            ) : (
              <Gauge className="h-4 w-4 text-amber-600" />
            )}
            <div>
              <div className="font-medium">{backendLabel(backend)}</div>
              <div className="text-sm text-muted-foreground">
                {backend === 'edgeparse'
                  ? t('settings.pdfParser.edgeparseHint', 'Fast, CPU-only, no API key required')
                  : t('settings.pdfParser.visionHint', 'Best for scanned and image-heavy PDFs')}
              </div>
            </div>
            <Badge variant="outline" className="ml-auto">
              {backend === 'none'
                ? t('settings.pdfParser.fallbackVision', 'Fallback: Vision')
                : backendLabel(backend)}
            </Badge>
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
