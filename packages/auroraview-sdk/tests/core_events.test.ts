/**
 * AuroraView SDK core/events.ts Tests
 *
 * Tests the EventEmitter class and getGlobalEmitter factory:
 * - on() / once() / off() / emit()
 * - hasHandlers() / handlerCount() / clear()
 * - Unsubscribe function behaviour
 * - getGlobalEmitter singleton semantics
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter, getGlobalEmitter } from '../src/core/events';

// ============================================================================
// Helpers
// ============================================================================

/** Reset the module-level singleton between tests */
function freshEmitter(): EventEmitter {
  return new EventEmitter();
}

// ============================================================================
// EventEmitter – on / emit
// ============================================================================

describe('EventEmitter.on / emit', () => {
  it('calls handler when event is emitted', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    em.on('test', handler);
    em.emit('test', { value: 42 });
    expect(handler).toHaveBeenCalledOnce();
    expect(handler).toHaveBeenCalledWith({ value: 42 });
  });

  it('does not call handler for different event', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    em.on('foo', handler);
    em.emit('bar', null);
    expect(handler).not.toHaveBeenCalled();
  });

  it('calls multiple handlers for same event', () => {
    const em = freshEmitter();
    const h1 = vi.fn();
    const h2 = vi.fn();
    em.on('multi', h1);
    em.on('multi', h2);
    em.emit('multi', 'data');
    expect(h1).toHaveBeenCalledWith('data');
    expect(h2).toHaveBeenCalledWith('data');
  });

  it('emitting unknown event is a no-op', () => {
    const em = freshEmitter();
    expect(() => em.emit('never-registered', null)).not.toThrow();
  });

  it('handler errors do not prevent other handlers from running', () => {
    const em = freshEmitter();
    const bad = vi.fn().mockImplementation(() => { throw new Error('boom'); });
    const good = vi.fn();
    em.on('ev', bad);
    em.on('ev', good);
    expect(() => em.emit('ev', null)).not.toThrow();
    expect(good).toHaveBeenCalled();
  });

  it('handles null data', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    em.on('nullev', handler);
    em.emit('nullev', null);
    expect(handler).toHaveBeenCalledWith(null);
  });

  it('handles undefined data', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    em.on('undev', handler);
    em.emit('undev', undefined);
    expect(handler).toHaveBeenCalledWith(undefined);
  });

  it('registers same handler twice only once (Set dedup)', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    em.on('dup', handler);
    em.on('dup', handler);
    em.emit('dup', 1);
    // Set deduplication: called once
    expect(handler).toHaveBeenCalledTimes(1);
  });
});

// ============================================================================
// EventEmitter – Unsubscribe function
// ============================================================================

describe('EventEmitter unsubscribe', () => {
  it('unsubscribe stops future events', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    const unsub = em.on('ev', handler);
    em.emit('ev', 1);
    unsub();
    em.emit('ev', 2);
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it('unsubscribing twice is safe', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    const unsub = em.on('ev', handler);
    unsub();
    expect(() => unsub()).not.toThrow();
  });

  it('cleanup removes empty event key', () => {
    const em = freshEmitter();
    const unsub = em.on('ev', vi.fn());
    unsub();
    expect(em.hasHandlers('ev')).toBe(false);
    expect(em.handlerCount('ev')).toBe(0);
  });

  it('unsubscribing one of two leaves the other', () => {
    const em = freshEmitter();
    const h1 = vi.fn();
    const h2 = vi.fn();
    const unsub1 = em.on('ev', h1);
    em.on('ev', h2);
    unsub1();
    em.emit('ev', 'x');
    expect(h1).not.toHaveBeenCalled();
    expect(h2).toHaveBeenCalledWith('x');
  });
});

// ============================================================================
// EventEmitter – once
// ============================================================================

describe('EventEmitter.once', () => {
  it('fires handler exactly once', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    em.once('ev', handler);
    em.emit('ev', 'first');
    em.emit('ev', 'second');
    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith('first');
  });

  it('returns unsubscribe that can be called early', () => {
    const em = freshEmitter();
    const handler = vi.fn();
    const unsub = em.once('ev', handler);
    unsub();
    em.emit('ev', 'x');
    expect(handler).not.toHaveBeenCalled();
  });

  it('multiple once handlers each fire once', () => {
    const em = freshEmitter();
    const h1 = vi.fn();
    const h2 = vi.fn();
    em.once('ev', h1);
    em.once('ev', h2);
    em.emit('ev', 1);
    em.emit('ev', 2);
    expect(h1).toHaveBeenCalledTimes(1);
    expect(h2).toHaveBeenCalledTimes(1);
  });
});

// ============================================================================
// EventEmitter – off
// ============================================================================

describe('EventEmitter.off', () => {
  it('off with handler removes specific handler', () => {
    const em = freshEmitter();
    const h1 = vi.fn();
    const h2 = vi.fn();
    em.on('ev', h1);
    em.on('ev', h2);
    em.off('ev', h1);
    em.emit('ev', 'x');
    expect(h1).not.toHaveBeenCalled();
    expect(h2).toHaveBeenCalled();
  });

  it('off without handler removes all handlers for event', () => {
    const em = freshEmitter();
    const h1 = vi.fn();
    const h2 = vi.fn();
    em.on('ev', h1);
    em.on('ev', h2);
    em.off('ev');
    em.emit('ev', 'x');
    expect(h1).not.toHaveBeenCalled();
    expect(h2).not.toHaveBeenCalled();
  });

  it('off on non-existent event is safe', () => {
    const em = freshEmitter();
    expect(() => em.off('nonexistent')).not.toThrow();
  });

  it('off specific handler on non-existent event is safe', () => {
    const em = freshEmitter();
    expect(() => em.off('nonexistent', vi.fn())).not.toThrow();
  });
});

// ============================================================================
// EventEmitter – hasHandlers / handlerCount
// ============================================================================

describe('EventEmitter.hasHandlers / handlerCount', () => {
  it('hasHandlers returns false initially', () => {
    const em = freshEmitter();
    expect(em.hasHandlers('ev')).toBe(false);
  });

  it('hasHandlers returns true after adding handler', () => {
    const em = freshEmitter();
    em.on('ev', vi.fn());
    expect(em.hasHandlers('ev')).toBe(true);
  });

  it('handlerCount returns 0 for unknown event', () => {
    const em = freshEmitter();
    expect(em.handlerCount('ev')).toBe(0);
  });

  it('handlerCount increments correctly', () => {
    const em = freshEmitter();
    em.on('ev', vi.fn());
    em.on('ev', vi.fn());
    expect(em.handlerCount('ev')).toBe(2);
  });

  it('handlerCount decrements after off', () => {
    const em = freshEmitter();
    const h = vi.fn();
    em.on('ev', h);
    em.on('ev', vi.fn());
    em.off('ev', h);
    expect(em.handlerCount('ev')).toBe(1);
  });
});

// ============================================================================
// EventEmitter – clear
// ============================================================================

describe('EventEmitter.clear', () => {
  it('clear removes all handlers for all events', () => {
    const em = freshEmitter();
    const h1 = vi.fn();
    const h2 = vi.fn();
    em.on('ev1', h1);
    em.on('ev2', h2);
    em.clear();
    em.emit('ev1', null);
    em.emit('ev2', null);
    expect(h1).not.toHaveBeenCalled();
    expect(h2).not.toHaveBeenCalled();
  });

  it('clear on empty emitter is safe', () => {
    const em = freshEmitter();
    expect(() => em.clear()).not.toThrow();
  });

  it('clear then re-register works', () => {
    const em = freshEmitter();
    const h = vi.fn();
    em.on('ev', vi.fn());
    em.clear();
    em.on('ev', h);
    em.emit('ev', 'after');
    expect(h).toHaveBeenCalledWith('after');
  });
});

// ============================================================================
// getGlobalEmitter
// ============================================================================

describe('getGlobalEmitter', () => {
  it('returns an EventEmitter instance', () => {
    const emitter = getGlobalEmitter();
    expect(emitter).toBeInstanceOf(EventEmitter);
  });

  it('returns same instance on repeated calls', () => {
    const em1 = getGlobalEmitter();
    const em2 = getGlobalEmitter();
    expect(em1).toBe(em2);
  });

  it('events registered on global emitter fire correctly', () => {
    const em = getGlobalEmitter();
    const handler = vi.fn();
    const unsub = em.on('global_test_ev', handler);
    em.emit('global_test_ev', { ok: true });
    expect(handler).toHaveBeenCalledWith({ ok: true });
    unsub();
  });
});
