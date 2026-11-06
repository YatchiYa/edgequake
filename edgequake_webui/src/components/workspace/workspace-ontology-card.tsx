/**
 * @module WorkspaceOntologyCard
 * @description Workspace ontology (SPEC-184): view, import (file or workspace document),
 * review/edit as JSON, save, export, remove; plan and run graph realignment.
 *
 * Saving affects future extractions only. Existing data changes only through
 * "Realign graph" here or "Rebuild knowledge graph" in the actions card.
 */
'use client';

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import {
  deleteWorkspaceOntology,
  draftWorkspaceOntology,
  getDocument,
  getDocuments,
  getWorkspaceOntology,
  planWorkspaceOntology,
  realignWorkspaceOntology,
  saveWorkspaceOntology,
  type OntologyDraftRequest,
  type OntologyPlanResponse,
  type WorkspaceOntology,
} from '@/lib/api/edgequake';
import type { Workspace } from '@/types';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { BookMarked, Download, FileUp, FileText, GitCompareArrows, Loader2, Pencil, Trash2 } from 'lucide-react';
import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

const FILE_ACCEPT = '.json,.csv,.tsv,.txt,.md,.markdown';
const EMPTY: WorkspaceOntology = { name: '', entity_types: [], relation_types: [], terms: [] };
const MAX_REALIGN_ROUNDS = 25;

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export interface WorkspaceOntologyCardProps {
  workspace: Workspace;
}

export function WorkspaceOntologyCard({ workspace }: WorkspaceOntologyCardProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const workspaceId = workspace.id;
  const fileInput = useRef<HTMLInputElement>(null);

  const [editorOpen, setEditorOpen] = useState(false);
  const [editorText, setEditorText] = useState('');
  const [editorNotes, setEditorNotes] = useState<string[]>([]);
  const [editorError, setEditorError] = useState<string | null>(null);
  const [documentPickerOpen, setDocumentPickerOpen] = useState(false);
  const [confirm, setConfirm] = useState<'remove' | 'realign' | null>(null);
  const [plan, setPlan] = useState<OntologyPlanResponse | null>(null);

  const ontologyQuery = useQuery({
    queryKey: ['workspaceOntology', workspaceId],
    queryFn: () => getWorkspaceOntology(workspaceId),
    enabled: !!workspaceId,
    staleTime: 30_000,
  });
  const ontology = ontologyQuery.data?.ontology ?? null;
  const counts = ontologyQuery.data?.counts;

  const documentsQuery = useQuery({
    queryKey: ['workspaceOntologyDocuments', workspaceId],
    queryFn: () => getDocuments({ page: 1, page_size: 50, status: 'completed', sort_by: 'created_at', sort_order: 'desc' }),
    enabled: documentPickerOpen,
  });

  const refreshAfterSave = () => {
    void queryClient.invalidateQueries({ queryKey: ['workspaceOntology', workspaceId] });
    void queryClient.invalidateQueries({ queryKey: ['workspace'] });
    void queryClient.invalidateQueries({ queryKey: ['workspaces'] });
    setPlan(null);
  };

  const openEditor = (value: WorkspaceOntology, notes: string[] = []) => {
    setEditorText(JSON.stringify(value, null, 2));
    setEditorNotes(notes);
    setEditorError(null);
    setEditorOpen(true);
  };

  const draftMutation = useMutation({
    mutationFn: (request: OntologyDraftRequest) => draftWorkspaceOntology(workspaceId, request),
    onSuccess: (draft) => {
      const read =
        draft.format === 'llm'
          ? t('ontology.draftReadByModel', 'Read by the language model ({{count}} segments).', { count: draft.segments })
          : t('ontology.draftReadDirectly', 'Read directly ({{format}}).', { format: draft.format.toUpperCase() });
      openEditor(draft.ontology, [read, ...draft.warnings]);
    },
    onError: (error) => toast.error(t('ontology.draftFailed', 'Could not draft an ontology'), { description: errorMessage(error) }),
  });

  const saveMutation = useMutation({
    mutationFn: (value: WorkspaceOntology) => saveWorkspaceOntology(workspaceId, value),
    onSuccess: (saved) => {
      setEditorOpen(false);
      refreshAfterSave();
      toast.success(t('ontology.saved', 'Ontology saved'), {
        description:
          saved.warnings.length > 0
            ? saved.warnings.slice(0, 3).join(' · ')
            : t('ontology.savedHint', 'New extractions follow it. Check alignment to update existing entities.'),
      });
    },
    onError: (error) => setEditorError(errorMessage(error)),
  });

  const removeMutation = useMutation({
    mutationFn: () => deleteWorkspaceOntology(workspaceId),
    onSuccess: () => {
      refreshAfterSave();
      toast.success(t('ontology.removed', 'Ontology removed; the previous type lists are back.'));
    },
    onError: (error) => toast.error(t('ontology.removeFailed', 'Could not remove the ontology'), { description: errorMessage(error) }),
  });

  const planMutation = useMutation({
    mutationFn: () => planWorkspaceOntology(workspaceId),
    onSuccess: setPlan,
    onError: (error) => toast.error(t('ontology.planFailed', 'Could not compare with the graph'), { description: errorMessage(error) }),
  });

  const realignMutation = useMutation({
    mutationFn: async () => {
      let merged = 0;
      let retyped = 0;
      let failed = 0;
      for (let round = 0; round < MAX_REALIGN_ROUNDS; round++) {
        const r = await realignWorkspaceOntology(workspaceId, 200);
        merged += r.merged;
        retyped += r.retyped;
        failed += r.failed;
        if (r.remaining === 0 || r.merged + r.retyped === 0) break;
      }
      return { merged, retyped, failed };
    },
    onSuccess: ({ merged, retyped, failed }) => {
      void queryClient.invalidateQueries({ queryKey: ['workspaceStats', workspaceId] });
      void queryClient.invalidateQueries({ queryKey: ['graph'] });
      const summary = t('ontology.realignDone', '{{merged}} merged, {{retyped}} retyped', { merged, retyped });
      if (failed > 0) {
        toast.warning(summary, { description: t('ontology.realignFailedSteps', '{{failed}} steps failed.', { failed }) });
      } else {
        toast.success(summary);
      }
      planMutation.mutate();
    },
    onError: (error) => toast.error(t('ontology.realignFailed', 'Realignment stopped'), { description: errorMessage(error) }),
  });

  const onFile = async (file: File) => {
    const text = await file.text();
    draftMutation.mutate({ text, file_name: file.name, source_kind: 'file' });
  };

  const onDocument = async (documentId: string) => {
    setDocumentPickerOpen(false);
    try {
      const document = await getDocument(documentId);
      if (!document.content?.trim()) {
        toast.error(t('ontology.documentEmpty', 'This document has no text yet.'));
        return;
      }
      draftMutation.mutate({
        text: document.content,
        file_name: document.file_name ?? document.title ?? documentId,
        document_id: documentId,
        source_kind: 'document',
      });
    } catch (error) {
      toast.error(t('ontology.documentFailed', 'Could not read the document'), { description: errorMessage(error) });
    }
  };

  const onSave = () => {
    let parsed: WorkspaceOntology;
    try {
      parsed = JSON.parse(editorText) as WorkspaceOntology;
    } catch (error) {
      setEditorError(t('ontology.invalidJson', 'Not valid JSON: {{message}}', { message: errorMessage(error) }));
      return;
    }
    setEditorError(null);
    saveMutation.mutate(parsed);
  };

  const onExport = () => {
    if (!ontology) return;
    const blob = new Blob([JSON.stringify(ontology, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${(ontology.name || 'ontology').replace(/[^\w.-]+/g, '_')}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const busy = draftMutation.isPending || saveMutation.isPending || removeMutation.isPending || realignMutation.isPending;
  const pendingSteps = plan ? plan.plan.merges.length + plan.plan.retypes.length : 0;

  return (
    <Card className="gap-2 py-4" data-testid="workspace-ontology-card">
      <CardHeader className="gap-1 px-4 pb-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <BookMarked className="h-4 w-4 text-violet-600" />
          {t('ontology.title', 'Ontology')}
          {ontology ? (
            <Badge variant="secondary" className="text-xs font-normal" data-testid="ontology-name">
              {ontology.name || t('ontology.unnamed', 'Unnamed')}
            </Badge>
          ) : null}
        </CardTitle>
        <CardDescription className="text-xs leading-snug">
          {t(
            'ontology.description',
            "Your organization's types, relations and reference terms guide extraction: definitions go to the model, and aliases become one canonical entity. Applies to future ingestions; realign or rebuild to update existing data.",
          )}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3 px-4">
        <input
          ref={fileInput}
          type="file"
          accept={FILE_ACCEPT}
          className="hidden"
          data-testid="ontology-file-input"
          onChange={(e) => {
            const file = e.target.files?.[0];
            e.target.value = '';
            if (file) void onFile(file);
          }}
        />

        {ontologyQuery.isLoading ? (
          <p className="text-xs text-muted-foreground">{t('common.loading', 'Loading…')}</p>
        ) : ontology && counts ? (
          <div className="space-y-2" data-testid="ontology-summary">
            <div className="flex flex-wrap gap-1.5 text-xs">
              <Badge variant="outline">{t('ontology.countTypes', '{{count}} entity types', { count: counts.entity_types })}</Badge>
              <Badge variant="outline">{t('ontology.countRelations', '{{count}} relation types', { count: counts.relation_types })}</Badge>
              <Badge variant="outline">{t('ontology.countTerms', '{{count}} terms', { count: counts.terms })}</Badge>
              <Badge variant="outline">{t('ontology.countAliases', '{{count}} aliases', { count: counts.aliases })}</Badge>
            </div>
            {ontology.entity_types.length > 0 ? (
              <div className="flex flex-wrap gap-1.5">
                {ontology.entity_types.slice(0, 24).map((type) => (
                  <Badge key={type.id} variant="secondary" className="font-mono text-xs" title={type.description || undefined}>
                    {type.id}
                    {type.parent ? <span className="ml-1 text-muted-foreground">⊂ {type.parent}</span> : null}
                  </Badge>
                ))}
              </div>
            ) : null}
            {ontology.terms.length > 0 ? (
              <ul className="space-y-0.5 text-xs text-muted-foreground" data-testid="ontology-terms">
                {ontology.terms.slice(0, 6).map((term) => (
                  <li key={term.name}>
                    <span className="font-medium text-foreground">{term.name}</span>
                    {term.entity_type ? <span className="font-mono"> · {term.entity_type}</span> : null}
                    {term.aliases?.length ? <span> — {term.aliases.join(', ')}</span> : null}
                  </li>
                ))}
                {ontology.terms.length > 6 ? (
                  <li className="italic">{t('ontology.moreTerms', '+{{count}} more', { count: ontology.terms.length - 6 })}</li>
                ) : null}
              </ul>
            ) : null}
            {ontology.sources?.length ? (
              <p className="text-xs text-muted-foreground">
                {t('ontology.sources', 'Sources')}: {ontology.sources.map((s) => s.name || s.kind).join(', ')}
              </p>
            ) : null}
          </div>
        ) : (
          <p
            className="rounded-md border border-dashed bg-muted/20 px-2.5 py-2 text-xs text-muted-foreground"
            data-testid="ontology-empty"
          >
            {t(
              'ontology.empty',
              'No ontology. Import your reference (JSON, CSV, text — or a PDF/DOCX already uploaded as a document), review it, then save.',
            )}
          </p>
        )}

        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="outline" disabled={busy} onClick={() => fileInput.current?.click()} data-testid="ontology-import-file">
            {draftMutation.isPending ? <Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" /> : <FileUp className="mr-1 h-3.5 w-3.5" />}
            {t('ontology.importFile', 'Import a file')}
          </Button>
          <Button size="sm" variant="outline" disabled={busy} onClick={() => setDocumentPickerOpen(true)} data-testid="ontology-import-document">
            <FileText className="mr-1 h-3.5 w-3.5" />
            {t('ontology.fromDocument', 'From a document')}
          </Button>
          <Button size="sm" variant="outline" disabled={busy} onClick={() => openEditor(ontology ?? EMPTY)} data-testid="ontology-edit">
            <Pencil className="mr-1 h-3.5 w-3.5" />
            {ontology ? t('ontology.edit', 'Edit') : t('ontology.writeJson', 'Write JSON')}
          </Button>
          {ontology ? (
            <>
              <Button size="sm" variant="outline" disabled={busy || planMutation.isPending} onClick={() => planMutation.mutate()} data-testid="ontology-plan">
                {planMutation.isPending ? <Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" /> : <GitCompareArrows className="mr-1 h-3.5 w-3.5" />}
                {t('ontology.checkAlignment', 'Check alignment')}
              </Button>
              <Button size="sm" variant="ghost" onClick={onExport} data-testid="ontology-export">
                <Download className="mr-1 h-3.5 w-3.5" />
                {t('ontology.export', 'Export')}
              </Button>
              <Button size="sm" variant="ghost" className="text-destructive" disabled={busy} onClick={() => setConfirm('remove')} data-testid="ontology-remove">
                <Trash2 className="mr-1 h-3.5 w-3.5" />
                {t('ontology.remove', 'Remove')}
              </Button>
            </>
          ) : null}
        </div>

        {plan ? (
          <div className="space-y-2 rounded-md border bg-muted/10 px-3 py-2 text-xs" data-testid="ontology-plan-result">
            <p>
              {t('ontology.planTerms', '{{found}} of {{total}} reference terms are in the graph.', {
                found: plan.plan.terms_in_graph,
                total: plan.plan.terms_total,
              })}{' '}
              {plan.entities_total > 0 && ontology?.entity_types.length
                ? t('ontology.planTypes', '{{percent}}% of entities have an ontology type.', {
                    percent: Math.round((100 * plan.entities_in_ontology_types) / plan.entities_total),
                  })
                : null}
            </p>
            {pendingSteps > 0 ? (
              <>
                <p className="font-medium">
                  {t('ontology.planSteps', 'Realign: {{merges}} alias entities to merge, {{retypes}} entities to retype.', {
                    merges: plan.plan.merges.length,
                    retypes: plan.plan.retypes.length,
                  })}
                </p>
                <ul className="max-h-32 space-y-0.5 overflow-auto font-mono text-muted-foreground">
                  {plan.plan.merges.slice(0, 20).map((m) => (
                    <li key={`m-${m.source}`}>
                      {m.source} → {m.target}
                      {m.target_exists ? '' : ` (${t('ontology.planCreated', 'created')})`}
                    </li>
                  ))}
                  {plan.plan.retypes.slice(0, 20).map((r) => (
                    <li key={`r-${r.entity}`}>
                      {r.entity}: {r.from} → {r.to}
                    </li>
                  ))}
                </ul>
                <Button size="sm" disabled={busy} onClick={() => setConfirm('realign')} data-testid="ontology-realign">
                  {realignMutation.isPending ? <Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" /> : null}
                  {t('ontology.realign', 'Realign graph')}
                </Button>
              </>
            ) : (
              <p className="text-muted-foreground">{t('ontology.planNothing', 'Nothing to merge or retype.')}</p>
            )}
            <p className="text-muted-foreground">
              {t(
                'ontology.rebuildHint',
                'Realigning fixes names and types already in the graph. To re-extract the {{count}} documents with the ontology (new relations, better descriptions), use Rebuild Knowledge Graph.',
                { count: plan.documents },
              )}
            </p>
          </div>
        ) : null}
      </CardContent>

      <Dialog open={editorOpen} onOpenChange={setEditorOpen}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>{t('ontology.editorTitle', 'Review the ontology')}</DialogTitle>
            <DialogDescription>
              {t(
                'ontology.editorDescription',
                'Check and correct before saving. entity_types: {id, description, parent, synonyms}; relation_types: {id, description, source_types, target_types}; terms: {name, entity_type, aliases, description}.',
              )}
            </DialogDescription>
          </DialogHeader>
          {editorNotes.length > 0 ? (
            <ul className="max-h-28 list-disc space-y-0.5 overflow-auto pl-5 text-xs text-muted-foreground" data-testid="ontology-editor-notes">
              {editorNotes.map((note, i) => (
                <li key={i}>{note}</li>
              ))}
            </ul>
          ) : null}
          <Textarea
            value={editorText}
            onChange={(e) => setEditorText(e.target.value)}
            className="h-[50vh] font-mono text-xs"
            spellCheck={false}
            data-testid="ontology-editor"
          />
          {editorError ? (
            <p className="whitespace-pre-wrap text-xs text-destructive" data-testid="ontology-editor-error">
              {editorError}
            </p>
          ) : null}
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditorOpen(false)}>
              {t('common.cancel', 'Cancel')}
            </Button>
            <Button onClick={onSave} disabled={saveMutation.isPending} data-testid="ontology-save">
              {saveMutation.isPending ? <Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" /> : null}
              {t('ontology.save', 'Save ontology')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={documentPickerOpen} onOpenChange={setDocumentPickerOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('ontology.pickDocument', 'Draft from a workspace document')}</DialogTitle>
            <DialogDescription>
              {t('ontology.pickDocumentHint', 'Upload your PDF or DOCX reference as a document first; its converted text is read here.')}
            </DialogDescription>
          </DialogHeader>
          <ul className="max-h-[50vh] space-y-1 overflow-auto text-sm" data-testid="ontology-document-list">
            {documentsQuery.isLoading ? <li className="text-muted-foreground">{t('common.loading', 'Loading…')}</li> : null}
            {documentsQuery.data?.items.map((doc) => (
              <li key={doc.id}>
                <button
                  type="button"
                  className="w-full truncate rounded px-2 py-1 text-left hover:bg-muted"
                  onClick={() => void onDocument(doc.id)}
                >
                  {doc.title || doc.file_name || doc.id}
                </button>
              </li>
            ))}
            {documentsQuery.data && documentsQuery.data.items.length === 0 ? (
              <li className="text-muted-foreground">{t('ontology.noDocuments', 'No processed documents yet.')}</li>
            ) : null}
          </ul>
        </DialogContent>
      </Dialog>

      <AlertDialog open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {confirm === 'remove' ? t('ontology.removeTitle', 'Remove the ontology?') : t('ontology.realignTitle', 'Realign the graph?')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {confirm === 'remove'
                ? t(
                    'ontology.removeDescription',
                    'Future extractions go back to the type lists the workspace had before. Entities already in the graph stay as they are.',
                  )
                : t(
                    'ontology.realignDescription',
                    'Alias entities are merged into their canonical entity (relationships move with them) and reference terms get their ontology type. Merges cannot be undone automatically.',
                  )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel', 'Cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (confirm === 'remove') removeMutation.mutate();
                else realignMutation.mutate();
                setConfirm(null);
              }}
            >
              {confirm === 'remove' ? t('ontology.remove', 'Remove') : t('ontology.realign', 'Realign graph')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  );
}
