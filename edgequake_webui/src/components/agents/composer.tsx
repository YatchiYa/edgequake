'use client';

import { Button } from '@/components/ui/button';
import { useAutoResize } from '@/hooks/use-auto-resize';
import { cn } from '@/lib/utils';
import { ArrowUp, Square } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

const MAX_CHARS = 8_000; // service caps requests at 32 KB (UTF-8 + JSON)

interface ComposerProps {
  onSend: (text: string) => void;
  onStop?: () => void;
  streaming?: boolean;
  disabled?: boolean;
  placeholder?: string;
  autoFocus?: boolean;
  className?: string;
}

/** Message box: Enter sends, Shift+Enter new line; Esc stops (handled globally by the chat view). */
export function Composer({ onSend, onStop, streaming, disabled, placeholder, autoFocus, className }: ComposerProps) {
  const { t } = useTranslation();
  const [value, setValue] = useState('');
  const ref = useRef<HTMLTextAreaElement>(null);
  useAutoResize(ref, value, 24, 220);
  useEffect(() => {
    if (autoFocus) ref.current?.focus();
  }, [autoFocus]);

  const canSend = !!value.trim() && !streaming && !disabled;
  const submit = () => {
    if (!canSend) return;
    onSend(value.trim());
    setValue('');
  };

  return (
    <form
      className={cn(
        'flex items-end gap-2 rounded-3xl border bg-card p-2 pl-4 shadow-sm transition-shadow focus-within:shadow-md focus-within:ring-1 focus-within:ring-ring/30',
        className,
      )}
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <textarea
        ref={ref}
        rows={1}
        value={value}
        maxLength={MAX_CHARS}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
            e.preventDefault();
            submit();
          }
        }}
        placeholder={placeholder ?? t('agentsPlatform.composer.placeholder')}
        aria-label={t('agentsPlatform.composer.label')}
        disabled={disabled}
        className="query-input max-h-[220px] min-h-6 flex-1 resize-none bg-transparent! py-1.5 text-[15px] leading-6 placeholder:text-muted-foreground disabled:opacity-60"
      />
      {streaming ? (
        <Button
          type="button"
          size="icon"
          className="size-9 rounded-full"
          onClick={onStop}
          aria-label={t('agentsPlatform.composer.stop')}
          title={t('agentsPlatform.composer.stopHint')}
        >
          <Square className="size-3.5 fill-current" />
        </Button>
      ) : (
        <Button type="submit" size="icon" className="size-9 rounded-full" disabled={!canSend} aria-label={t('agentsPlatform.composer.send')}>
          <ArrowUp className="size-4" />
        </Button>
      )}
    </form>
  );
}
