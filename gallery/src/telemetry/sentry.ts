import * as Sentry from '@sentry/react'
import { resourceFromAttributes } from '@opentelemetry/resources'
import {
  SEMRESATTRS_SERVICE_NAME,
  SEMRESATTRS_SERVICE_VERSION,
  SEMRESATTRS_DEPLOYMENT_ENVIRONMENT,
} from '@opentelemetry/semantic-conventions'
import { WebTracerProvider } from '@opentelemetry/sdk-trace-web'
import { BatchSpanProcessor, ParentBasedSampler, TraceIdRatioBasedSampler } from '@opentelemetry/sdk-trace-base'

import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http'
import { registerInstrumentations } from '@opentelemetry/instrumentation'
import { DocumentLoadInstrumentation } from '@opentelemetry/instrumentation-document-load'
import { FetchInstrumentation } from '@opentelemetry/instrumentation-fetch'
import { XMLHttpRequestInstrumentation } from '@opentelemetry/instrumentation-xml-http-request'

// Default values are empty - DSN must be provided via environment variables
// Local dev: gallery/.env.sentry (loaded by Vite)
// CI/CD: GitHub Secrets injected at build time
const DEFAULT_FRONTEND_SENTRY_DSN = ''
const DEFAULT_FRONTEND_OTLP_ENDPOINT = ''

let sentryInitialized = false

function isDisabled(value: string | undefined): boolean {

  if (!value) {
    return false
  }
  const lowered = value.toLowerCase()
  return lowered === '1' || lowered === 'true' || lowered === 'yes' || lowered === 'on'
}

function toRate(value: string | undefined, fallback: number): number {
  if (!value) {
    return fallback
  }
  const parsed = Number(value)
  if (Number.isFinite(parsed) && parsed >= 0 && parsed <= 1) {
    return parsed
  }
  return fallback
}

function buildOtlpTracesUrl(endpoint: string): string {
  const normalized = endpoint.replace(/\/+$/, '')
  if (normalized.endsWith('/v1/traces')) {
    return normalized
  }
  return `${normalized}/v1/traces`
}

function initSentry(): void {
  const disabled = isDisabled(import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DISABLED)
  if (disabled) {
    return
  }

  const dsn = import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DSN ?? DEFAULT_FRONTEND_SENTRY_DSN
  const otlpEndpoint =
    import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_OTLP_ENDPOINT ??
    DEFAULT_FRONTEND_OTLP_ENDPOINT

  // Skip Sentry initialization if DSN is not configured
  if (!dsn) {
    console.info('[AuroraView] Sentry DSN not configured, skipping telemetry initialization')
    return
  }

  Sentry.init({
    dsn,
    environment: import.meta.env.MODE,
    sampleRate: toRate(import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_SAMPLE_RATE, 1.0),
    tracesSampleRate: toRate(
      import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_TRACES_SAMPLE_RATE,
      0.2,
    ),
    initialScope: {
      tags: {
        app: 'auroraview-gallery-frontend',
      },
      contexts: {
        telemetry: {
          otlp_endpoint: otlpEndpoint,
        },
      },
    },
  })

  sentryInitialized = true
}


function initOtel(): void {
  const disabled = isDisabled(import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_OTEL_DISABLED)
  if (disabled) {
    return
  }

  const otlpEndpoint =
    import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_OTLP_ENDPOINT ??
    DEFAULT_FRONTEND_OTLP_ENDPOINT

  // Skip OTel initialization if endpoint is not configured
  if (!otlpEndpoint) {
    console.info('[AuroraView] OTLP endpoint not configured, skipping OpenTelemetry initialization')
    return
  }

  const traceSampleRatio = toRate(
    import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_OTEL_TRACE_SAMPLE_RATIO,
    0.2,
  )

  const serviceName =
    import.meta.env.VITE_AURORAVIEW_GALLERY_FRONTEND_OTEL_SERVICE_NAME ??
    'auroraview-gallery-frontend'

  const serviceVersion = import.meta.env.VITE_APP_VERSION ?? '0.1.0'

  const traceExporter = new OTLPTraceExporter({
    url: buildOtlpTracesUrl(otlpEndpoint),
  })

  const provider = new WebTracerProvider({
    resource: resourceFromAttributes({
      [SEMRESATTRS_SERVICE_NAME]: serviceName,
      [SEMRESATTRS_SERVICE_VERSION]: serviceVersion,
      [SEMRESATTRS_DEPLOYMENT_ENVIRONMENT]: import.meta.env.MODE,
    }),
    sampler: new ParentBasedSampler({
      root: new TraceIdRatioBasedSampler(traceSampleRatio),
    }),
    spanProcessors: [new BatchSpanProcessor(traceExporter)],
  })

  provider.register()


  registerInstrumentations({
    instrumentations: [
      new DocumentLoadInstrumentation(),
      new FetchInstrumentation({
        propagateTraceHeaderCorsUrls: /.*/,
      }),
      new XMLHttpRequestInstrumentation({
        propagateTraceHeaderCorsUrls: /.*/,
      }),
    ],
  })
}

function initPackedRuntimeEventHooks(): void {
  window.addEventListener('auroraview:backend_error', (event: Event) => {
    const payload = (event as CustomEvent<{ message?: string; source?: string }>).detail ?? {}
    const message = payload.message ?? 'unknown backend error'
    const source = payload.source ?? 'unknown'
    Sentry.captureMessage(`[packed.backend_error][${source}] ${message}`, 'error')
  })

  window.addEventListener('auroraview:packed_startup_metrics', (event: Event) => {
    const payload = (event as CustomEvent<Record<string, unknown>>).detail ?? {}
    Sentry.captureMessage('[packed.startup_metrics] startup telemetry received', 'info')
    Sentry.setContext('packed_startup_metrics', payload)
  })
}

export function triggerSentryTestEvent(): {
  ok: boolean
  eventId?: string
  reason?: string
} {
  if (!sentryInitialized) {
    return {
      ok: false,
      reason:
        'Sentry 未初始化（请检查 VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DSN 或禁用开关）',
    }
  }

  const now = new Date().toISOString()
  const message = `[gallery.sentry_test] manual exception trigger at ${now}`

  const eventId = Sentry.captureException(new Error(message), {
    tags: {
      source: 'gallery.telemetry_panel',
      trigger: 'manual',
    },
    level: 'error',
  })

  Sentry.captureMessage(`[gallery.sentry_test] manual breadcrumb at ${now}`, 'info')

  return {
    ok: true,
    eventId: eventId || undefined,
  }
}

export function triggerSentryPromiseRejectionTestEvent(): {
  ok: boolean
  reason?: string
} {
  if (!sentryInitialized) {
    return {
      ok: false,
      reason:
        'Sentry 未初始化（请检查 VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DSN 或禁用开关）',
    }
  }

  const now = new Date().toISOString()
  const message = `[gallery.sentry_test] manual unhandled rejection trigger at ${now}`

  window.setTimeout(() => {
    Promise.reject(new Error(message))
  }, 0)

  Sentry.captureMessage(`[gallery.sentry_test] scheduled unhandled rejection at ${now}`, 'info')

  return {
    ok: true,
  }
}

initSentry()
initOtel()
initPackedRuntimeEventHooks()
