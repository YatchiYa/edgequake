'use client';

/**
 * Knowledge Injection Page (SPEC-0002)
 *
 * Allows users to inject domain glossaries, acronyms, and definitions
 * to enrich the knowledge graph without polluting query citations.
 */

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
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Textarea } from '@/components/ui/textarea';
import {
    useCreateInjection,
    useDeleteInjection,
    useInjections,
} from '@/hooks';
import useTenantContext from '@/hooks/use-tenant-context';
import { BookOpen, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';

export default function KnowledgePage() {
  const { selectedWorkspaceId } = useTenantContext();
  const { data, isLoading } = useInjections(selectedWorkspaceId);
  const createMutation = useCreateInjection(selectedWorkspaceId ?? '');
  const deleteMutation = useDeleteInjection(selectedWorkspaceId ?? '');

  const [dialogOpen, setDialogOpen] = useState(false);
  const [name, setName] = useState('');
  const [content, setContent] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!name.trim() || !content.trim()) {
      toast.error('Name and content are required');
      return;
    }
    try {
      await createMutation.mutateAsync({ name: name.trim(), content: content.trim() });
      toast.success('Knowledge injection created');
      setDialogOpen(false);
      setName('');
      setContent('');
    } catch (err) {
      toast.error(`Failed to create injection: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const handleDelete = async (injectionId: string) => {
    try {
      await deleteMutation.mutateAsync(injectionId);
      toast.success('Injection deleted');
      setDeleteTarget(null);
    } catch (err) {
      toast.error(`Failed to delete: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const statusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'default';
      case 'processing': return 'secondary';
      case 'failed': return 'destructive';
      default: return 'outline';
    }
  };

  return (
    <div className="flex flex-col gap-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <BookOpen className="h-6 w-6" />
            Knowledge Injection
          </h1>
          <p className="text-muted-foreground mt-1">
            Inject domain glossaries, acronyms, and definitions to enrich your knowledge graph.
          </p>
        </div>

        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-4 w-4 mr-2" />
              New Injection
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>Create Knowledge Injection</DialogTitle>
              <DialogDescription>
                Paste domain glossary, acronym definitions, or background knowledge.
                Entities will be extracted and merged into the knowledge graph.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="injection-name">Name</Label>
                <Input
                  id="injection-name"
                  placeholder="e.g., Manufacturing Glossary"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  maxLength={100}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="injection-content">Content</Label>
                <Textarea
                  id="injection-content"
                  placeholder="OEE: Overall Equipment Effectiveness, a measure of manufacturing productivity&#10;MTBF: Mean Time Between Failures&#10;..."
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  rows={12}
                  className="font-mono text-sm"
                />
                <p className="text-xs text-muted-foreground">
                  {content.length.toLocaleString()} / 102,400 characters
                </p>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setDialogOpen(false)}>
                Cancel
              </Button>
              <Button
                onClick={handleCreate}
                disabled={createMutation.isPending || !name.trim() || !content.trim()}
              >
                {createMutation.isPending ? 'Creating...' : 'Create'}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {isLoading ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardHeader>
                <Skeleton className="h-5 w-40" />
                <Skeleton className="h-4 w-24" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-4 w-full" />
              </CardContent>
            </Card>
          ))}
        </div>
      ) : !data?.items?.length ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-12">
            <BookOpen className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-1">No knowledge injections yet</h3>
            <p className="text-muted-foreground text-center max-w-md">
              Inject domain-specific glossaries and definitions to improve search quality.
              Injected knowledge enriches the graph but won&apos;t appear in citations.
            </p>
            <Button className="mt-4" onClick={() => setDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              Create your first injection
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {data.items.map((item) => (
            <Card key={item.injection_id}>
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <CardTitle className="text-base">{item.name}</CardTitle>
                  <Badge variant={statusColor(item.status)}>{item.status}</Badge>
                </div>
                <CardDescription className="text-xs">
                  {item.entity_count} entities &middot; {item.source_type}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>
                    {new Date(item.created_at).toLocaleDateString()}
                  </span>
                  <Dialog
                    open={deleteTarget === item.injection_id}
                    onOpenChange={(open) => setDeleteTarget(open ? item.injection_id : null)}
                  >
                    <DialogTrigger asChild>
                      <Button variant="ghost" size="icon" className="h-7 w-7">
                        <Trash2 className="h-3.5 w-3.5 text-destructive" />
                      </Button>
                    </DialogTrigger>
                    <DialogContent>
                      <DialogHeader>
                        <DialogTitle>Delete &ldquo;{item.name}&rdquo;?</DialogTitle>
                        <DialogDescription>
                          This will remove the injection and its extracted entities from the knowledge graph.
                        </DialogDescription>
                      </DialogHeader>
                      <DialogFooter>
                        <Button variant="outline" onClick={() => setDeleteTarget(null)}>
                          Cancel
                        </Button>
                        <Button
                          variant="destructive"
                          onClick={() => handleDelete(item.injection_id)}
                          disabled={deleteMutation.isPending}
                        >
                          {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
                        </Button>
                      </DialogFooter>
                    </DialogContent>
                  </Dialog>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
