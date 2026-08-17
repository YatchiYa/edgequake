'use client';

<<<<<<< HEAD
import { formatEntityLabel, formatEntityType, getEntityTypeColor } from '@/lib/graph/label-utils';
import type { GraphNode } from '@/types';
=======
/**
 * @module NodeContextMenu
 * @description Context menu for a graph node, opened by a canvas right-click at
 * the cursor position.
 *
 * Built on the Radix `DropdownMenu` primitive (DRY with the rest of the app) so
 * keyboard navigation, focus management, Escape-to-close, and click-outside are
 * inherited rather than hand-rolled:
 *  - ArrowUp/Down + Home/End + type-ahead move between items.
 *  - Enter/Space selects; Escape closes; focus is trapped while open.
 *  - Viewport collision (flip/clamp) is handled by Radix Popper — no manual
 *    coordinate clamping.
 *
 * Because the menu is opened programmatically from a canvas (not a DOM trigger),
 * a 1px `Anchor` is placed at the cursor to position the content.
 */

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuShortcut,
} from '@/components/ui/dropdown-menu';
import { useEntityTypeColors } from '@/hooks/use-entity-type-colors';
import { formatEntityLabel, formatEntityType } from '@/lib/graph/label-utils';
import type { GraphNode } from '@/types';
import * as DropdownMenuPrimitive from '@radix-ui/react-dropdown-menu';
import {
    Check,
    Copy,
    Eye,
    FileText,
    Minimize2,
    Network,
    Search,
    Trash2
} from 'lucide-react';
<<<<<<< HEAD
import { useCallback, useEffect, useRef, useState } from 'react';
=======
import { useCallback, useState } from 'react';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { useTranslation } from 'react-i18next';

interface NodeContextMenuPosition {
  x: number;
  y: number;
}

interface NodeContextMenuProps {
  node: GraphNode | null;
  position: NodeContextMenuPosition | null;
  onClose: () => void;
  onViewDetails: (node: GraphNode) => void;
  onExpandNeighborhood: (node: GraphNode) => void;
  onPruneNode?: (node: GraphNode) => void;
  onFindRelated: (node: GraphNode) => void;
  onViewDocuments: (node: GraphNode) => void;
  onCopyId: (node: GraphNode) => void;
  onDelete?: (node: GraphNode) => void;
  isExpanded?: boolean;
}

export function NodeContextMenu({
  node,
  position,
  onClose,
  onViewDetails,
  onExpandNeighborhood,
  onPruneNode,
  onFindRelated,
  onViewDocuments,
  onCopyId,
  onDelete,
  isExpanded = false,
}: NodeContextMenuProps) {
  const { t } = useTranslation();
<<<<<<< HEAD
  const menuRef = useRef<HTMLDivElement>(null);
  // Two-step delete confirm — avoids accidental deletion without a full modal
  const [pendingDelete, setPendingDelete] = useState(false);

  const handleClose = useCallback(() => {
    setPendingDelete(false);
    onClose();
  }, [onClose]);
=======
  // Two-step delete confirm — avoids accidental deletion without a full modal
  const [pendingDelete, setPendingDelete] = useState(false);

  const open = Boolean(node && position);

  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) {
        setPendingDelete(false);
        onClose();
      }
    },
    [onClose],
  );

  const { colorFor } = useEntityTypeColors();
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

  // Compute final safe position for the context menu.
  // position.x/y = absolute screen coords of the right-click cursor (on the node).
  // Push the menu to the right by a small offset so it sits beside the node,
  // center it vertically on the click point, clamp to viewport bounds.
  const safePos = useCallback(() => {
    if (!position) return { left: 0, top: 0 };
    const W = window.innerWidth;
    const H = window.innerHeight;
    const MENU_W = 240;
    const MENU_H = 340;
    const OFFSET_X = 8;  // push right from click point
    const rawLeft = position.x + OFFSET_X;
    const rawTop  = position.y - MENU_H / 2; // center vertically on click point
    return {
      left: Math.min(Math.max(rawLeft, 8), W - MENU_W - 8),
      top:  Math.min(Math.max(rawTop,  8), H - MENU_H - 8),
    };
  }, [position]);

  if (!node || !position) return null;

  const displayLabel = formatEntityLabel(node.label ?? '', 40);
<<<<<<< HEAD
  const displayType  = formatEntityType(node.node_type ?? '');
  const typeColor    = getEntityTypeColor(node.node_type);
  const pos = safePos();

  // Reusable menu item builder — keeps JSX DRY
  const Item = ({
    icon: Icon,
    label,
    kbd,
    danger = false,
    check,
    onClick,
  }: {
    icon: React.FC<{ className?: string }>;
    label: string;
    kbd?: string;
    danger?: boolean;
    check?: boolean;
    onClick: () => void;
  }) => (
    <button
      className={[
        'flex items-center gap-2 w-full px-2 py-1.5 text-sm rounded-sm transition-colors',
        danger
          ? 'text-destructive hover:bg-destructive/10 hover:text-destructive'
          : 'hover:bg-accent hover:text-accent-foreground',
      ].join(' ')}
      onClick={() => { onClick(); handleClose(); }}
    >
      <Icon className="h-4 w-4 shrink-0" />
      <span className="flex-1 text-left">{label}</span>
      {check && <span className="text-xs text-muted-foreground">✓</span>}
      {kbd && (
        <kbd className="ml-auto text-[10px] font-mono bg-muted/70 text-muted-foreground px-1.5 py-0.5 rounded border border-border/50">
          {kbd}
        </kbd>
      )}
    </button>
  );

  return (
    <div
      ref={menuRef}
      className="fixed z-50"
      style={{ left: pos.left, top: pos.top }}
    >
      <div className="bg-popover border rounded-lg shadow-lg p-1 min-w-60">
        {/* Header: formatted name + type with color dot */}
        <div className="px-2.5 py-2 border-b mb-1">
=======
  const displayType = formatEntityType(node.node_type ?? '');
  const typeColor = colorFor(node.node_type);

  return (
    <DropdownMenu open={open} onOpenChange={handleOpenChange}>
      {/*
        Virtual trigger at the right-click cursor. The menu is opened from the
        graph canvas, so there is no real DOM button; Radix positions the content
        against this 1px point and flips/clamps it within the viewport. It is
        pointer-events:none so it never intercepts interaction, and `open` is
        controlled externally so the trigger's built-in toggle never fires.
      */}
      <DropdownMenuPrimitive.Trigger asChild>
        <span
          aria-hidden
          tabIndex={-1}
          style={{
            position: 'fixed',
            left: position.x,
            top: position.y,
            width: 1,
            height: 1,
            pointerEvents: 'none',
          }}
        />
      </DropdownMenuPrimitive.Trigger>

      <DropdownMenuContent
        side="right"
        align="start"
        sideOffset={8}
        collisionPadding={8}
        className="min-w-60"
        data-testid="node-context-menu"
        // Focus returns to the canvas, not the 1px anchor, when the menu closes.
        onCloseAutoFocus={(event) => event.preventDefault()}
      >
        {/* Header: formatted name + type with color dot */}
        <DropdownMenuLabel className="font-normal" data-testid="node-context-menu-header">
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
          <div
            className="font-semibold text-sm truncate"
            title={formatEntityLabel(node.label ?? '', 200)}
          >
            {displayLabel}
          </div>
          <div className="flex items-center gap-1.5 mt-0.5">
            <span
              className="inline-block w-2 h-2 rounded-full shrink-0"
              style={{ backgroundColor: typeColor }}
              aria-hidden="true"
            />
            <span className="text-xs text-muted-foreground">{displayType}</span>
          </div>
<<<<<<< HEAD
        </div>

        <Item icon={Eye}     label={t('graph.contextMenu.viewDetails', 'View Details')}          kbd="↵"  onClick={() => onViewDetails(node)} />
        <Item icon={Network} label={t('graph.contextMenu.expandNeighborhood', 'Expand Neighborhood')} check={isExpanded} onClick={() => onExpandNeighborhood(node)} />
        {onPruneNode && (
          <Item icon={Minimize2} label={t('graph.contextMenu.pruneNode', 'Prune Node')} onClick={() => onPruneNode!(node)} />
        )}
        <Item icon={Search}  label={t('graph.contextMenu.findRelated', 'Find Related')}                       onClick={() => onFindRelated(node)} />

        <div className="my-1 h-px bg-border" />

        <Item icon={FileText} label={t('graph.contextMenu.viewDocuments', 'View Documents')}           onClick={() => onViewDocuments(node)} />
        <Item icon={Copy}     label={t('graph.contextMenu.copyId', 'Copy Entity ID')}        kbd="⌘C" onClick={() => onCopyId(node)} />

        {onDelete && (
          <>
            <div className="my-1 h-px bg-border" />
            {pendingDelete ? (
              // Step 2: inline confirm
              <div className="px-2 py-1.5">
                <p className="text-xs text-destructive font-medium mb-1.5">
                  {t('graph.contextMenu.deleteConfirm', 'Delete this entity?')}
                </p>
                <div className="flex gap-1.5">
                  <button
                    className="flex-1 text-xs px-2 py-1 rounded-sm bg-muted hover:bg-muted/80 transition-colors"
                    onClick={() => setPendingDelete(false)}
                  >
                    {t('common.cancel', 'Cancel')}
                  </button>
                  <button
                    className="flex-1 text-xs px-2 py-1 rounded-sm bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
                    onClick={() => { onDelete!(node); handleClose(); }}
                  >
                    {t('common.delete', 'Delete')}
                  </button>
                </div>
              </div>
            ) : (
              // Step 1: show Delete item
              <Item
                icon={Trash2}
                label={t('graph.contextMenu.deleteEntity', 'Delete Entity')}
                danger
                onClick={() => setPendingDelete(true)}
              />
=======
        </DropdownMenuLabel>
        <DropdownMenuSeparator />

        <DropdownMenuItem onSelect={() => onViewDetails(node)}>
          <Eye />
          <span className="flex-1">{t('graph.contextMenu.viewDetails', 'View Details')}</span>
          <DropdownMenuShortcut>↵</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => onExpandNeighborhood(node)}>
          <Network />
          <span className="flex-1">{t('graph.contextMenu.expandNeighborhood', 'Expand Neighborhood')}</span>
          {isExpanded && <Check className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />}
        </DropdownMenuItem>
        {onPruneNode && (
          <DropdownMenuItem onSelect={() => onPruneNode(node)}>
            <Minimize2 />
            <span className="flex-1">{t('graph.contextMenu.pruneNode', 'Prune Node')}</span>
          </DropdownMenuItem>
        )}
        <DropdownMenuItem onSelect={() => onFindRelated(node)}>
          <Search />
          <span className="flex-1">{t('graph.contextMenu.findRelated', 'Find Related')}</span>
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        <DropdownMenuItem onSelect={() => onViewDocuments(node)}>
          <FileText />
          <span className="flex-1">{t('graph.contextMenu.viewDocuments', 'View Documents')}</span>
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={() => onCopyId(node)}>
          <Copy />
          <span className="flex-1">{t('graph.contextMenu.copyId', 'Copy Entity ID')}</span>
          <DropdownMenuShortcut>⌘C</DropdownMenuShortcut>
        </DropdownMenuItem>

        {onDelete && (
          <>
            <DropdownMenuSeparator />
            {pendingDelete ? (
              <>
                <DropdownMenuLabel className="text-destructive text-xs font-medium">
                  {t('graph.contextMenu.deleteConfirm', 'Delete this entity?')}
                </DropdownMenuLabel>
                <DropdownMenuItem
                  // Keep the menu open so the user can confirm or cancel.
                  onSelect={(event) => {
                    event.preventDefault();
                    setPendingDelete(false);
                  }}
                >
                  <span className="flex-1">{t('common.cancel', 'Cancel')}</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  variant="destructive"
                  data-testid="node-context-menu-delete-confirm"
                  onSelect={() => onDelete(node)}
                >
                  <Trash2 />
                  <span className="flex-1">{t('common.delete', 'Delete')}</span>
                </DropdownMenuItem>
              </>
            ) : (
              <DropdownMenuItem
                variant="destructive"
                // Keep the menu open to reveal the inline confirm step.
                onSelect={(event) => {
                  event.preventDefault();
                  setPendingDelete(true);
                }}
              >
                <Trash2 />
                <span className="flex-1">{t('graph.contextMenu.deleteEntity', 'Delete Entity')}</span>
              </DropdownMenuItem>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            )}
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/**
 * Hook to manage context-menu open state. Open/close side effects (Escape,
 * click-outside) are handled by the Radix DropdownMenu via `onClose`, so this
 * hook only tracks which node + cursor position the menu is for.
 */
export function useNodeContextMenu() {
  const [contextMenuState, setContextMenuState] = useState<{
    node: GraphNode | null;
    position: { x: number; y: number } | null;
  }>({ node: null, position: null });

  const openContextMenu = useCallback((node: GraphNode, x: number, y: number) => {
    setContextMenuState({ node, position: { x, y } });
  }, []);

  const closeContextMenu = useCallback(() => {
    setContextMenuState({ node: null, position: null });
  }, []);

  return {
    contextMenuNode: contextMenuState.node,
    contextMenuPosition: contextMenuState.position,
    openContextMenu,
    closeContextMenu,
  };
}

export default NodeContextMenu;
