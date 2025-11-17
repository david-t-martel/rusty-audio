/**
 * Cloudflare Worker for Rusty Audio
 *
 * This worker serves the static WASM application with proper headers
 * for cross-origin isolation and multithreading support.
 */

import { getAssetFromKV } from '@cloudflare/kv-asset-handler';

/**
 * Handle incoming requests
 */
export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);

    try {
      // Health check endpoint
      if (url.pathname === '/health') {
        return new Response(JSON.stringify({
          status: 'ok',
          timestamp: new Date().toISOString(),
          environment: env.ENVIRONMENT || 'unknown',
          features: {
            multithreading: true,
            serviceWorker: true,
            wasm: true
          }
        }), {
          headers: {
            'Content-Type': 'application/json',
            'Cache-Control': 'no-cache'
          }
        });
      }

      // Features API endpoint
      if (url.pathname === '/api/features') {
        const features = await env.FEATURES?.get('enabled_features');
        return new Response(features || JSON.stringify({
          threading: { enabled: true, maxWorkers: 8 },
          wasm: { streaming: true, bulk_memory: true, simd: true, threads: true },
          browser: { crossOriginIsolated: true, sharedArrayBuffer: true }
        }), {
          headers: {
            'Content-Type': 'application/json',
            'Cache-Control': 'public, max-age=300'
          }
        });
      }

      // Get the asset from KV
      const options = {
        cacheControl: {
          browserTTL: 3600,
          edgeTTL: 7200,
          bypassCache: false
        }
      };

      const response = await getAssetFromKV(
        {
          request,
          waitUntil: ctx.waitUntil.bind(ctx)
        },
        options
      );

      // Clone the response to modify headers
      const modifiedResponse = new Response(response.body, response);

      // Set critical headers for multithreading
      modifiedResponse.headers.set('Cross-Origin-Opener-Policy', 'same-origin');
      modifiedResponse.headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
      modifiedResponse.headers.set('Cross-Origin-Resource-Policy', 'cross-origin');

      // Security headers
      modifiedResponse.headers.set('X-Content-Type-Options', 'nosniff');
      modifiedResponse.headers.set('X-Frame-Options', 'DENY');
      modifiedResponse.headers.set('Referrer-Policy', 'no-referrer');
      modifiedResponse.headers.set('Permissions-Policy', 'autoplay=(self), microphone=(self)');

      // WASM-specific headers
      if (url.pathname.endsWith('.wasm')) {
        modifiedResponse.headers.set('Content-Type', 'application/wasm');
        modifiedResponse.headers.set('Cache-Control', 'public, max-age=31536000, immutable');
      }

      // JavaScript module headers
      if (url.pathname.endsWith('.js')) {
        modifiedResponse.headers.set('Content-Type', 'application/javascript');
        if (!url.pathname.includes('service-worker')) {
          modifiedResponse.headers.set('Cache-Control', 'public, max-age=31536000, immutable');
        } else {
          modifiedResponse.headers.set('Cache-Control', 'no-store, no-cache, must-revalidate');
          modifiedResponse.headers.set('Service-Worker-Allowed', '/');
        }
      }

      return modifiedResponse;

    } catch (error) {
      // Asset not found or error occurred
      if (error.status === 404) {
        // SPA fallback - serve index.html
        const indexRequest = new Request(new URL('/index.html', request.url), request);

        try {
          const indexResponse = await getAssetFromKV({
            request: indexRequest,
            waitUntil: ctx.waitUntil.bind(ctx)
          });

          const response = new Response(indexResponse.body, indexResponse);
          response.headers.set('Cross-Origin-Opener-Policy', 'same-origin');
          response.headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
          response.headers.set('Cross-Origin-Resource-Policy', 'cross-origin');

          return response;
        } catch (e) {
          return new Response('Not Found', { status: 404 });
        }
      }

      return new Response(`Error: ${error.message}`, {
        status: error.status || 500,
        headers: {
          'Content-Type': 'text/plain'
        }
      });
    }
  }
};
