-- ============================================================================
-- Migration 115: SPEC-120 P1.5 — task status transition guard (INV-4)
-- Version: 1.0.0 — 2026-07-27
--
-- Mirrors Rust `TaskStatus::allows` for the absorbing terminal set and the
-- cancelling → cancelled drain path. Equal-state updates remain allowed
-- (idempotent persistence / recovery).
-- ============================================================================

SET search_path = public;

CREATE OR REPLACE FUNCTION edgequake_tasks_status_transition_ok(
    old_status TEXT,
    new_status TEXT
) RETURNS BOOLEAN
LANGUAGE plpgsql
IMMUTABLE
AS $$
BEGIN
    IF old_status IS NULL OR new_status IS NULL THEN
        RETURN FALSE;
    END IF;
    IF old_status = new_status THEN
        RETURN TRUE;
    END IF;

    -- Absorbing terminals (DeadLetter may return to Pending via explicit retry).
    IF old_status IN ('indexed', 'cancelled') THEN
        RETURN FALSE;
    END IF;
    IF old_status = 'dead_letter' THEN
        RETURN new_status = 'pending';
    END IF;

    IF old_status = 'cancelling' THEN
        RETURN new_status IN ('cancelled', 'failed', 'dead_letter');
    END IF;

    IF old_status = 'pending' THEN
        RETURN new_status IN (
            'held', 'processing', 'cancelling', 'cancelled',
            'indexed', 'failed', 'dead_letter'
        );
    END IF;

    IF old_status = 'held' THEN
        RETURN new_status IN ('pending', 'cancelling', 'cancelled', 'processing');
    END IF;

    IF old_status = 'processing' THEN
        RETURN new_status IN (
            'pending', 'held', 'cancelling', 'indexed',
            'failed', 'dead_letter', 'cancelled'
        );
    END IF;

    IF old_status = 'failed' THEN
        RETURN new_status IN ('pending', 'cancelling', 'cancelled', 'dead_letter');
    END IF;

    RETURN FALSE;
END;
$$;

CREATE OR REPLACE FUNCTION edgequake_tasks_enforce_status_transition()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_OP = 'UPDATE'
       AND NEW.status IS DISTINCT FROM OLD.status
       AND NOT edgequake_tasks_status_transition_ok(OLD.status, NEW.status)
    THEN
        RAISE EXCEPTION 'illegal task status transition: % -> % (track_id=%)',
            OLD.status, NEW.status, NEW.track_id
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_tasks_status_transition ON tasks;
CREATE TRIGGER trg_tasks_status_transition
    BEFORE UPDATE OF status ON tasks
    FOR EACH ROW
    EXECUTE FUNCTION edgequake_tasks_enforce_status_transition();

COMMENT ON FUNCTION edgequake_tasks_status_transition_ok(TEXT, TEXT) IS
    'SPEC-120 INV-4: mirrors Rust TaskStatus::allows for DB-side enforcement';
