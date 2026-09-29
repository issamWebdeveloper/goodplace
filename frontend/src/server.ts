import {
  AngularNodeAppEngine,
  createNodeRequestHandler,
  isMainModule,
  writeResponseToNodeResponse,
} from '@angular/ssr/node';
import express, { type Request, type Response } from 'express';
import { Readable } from 'node:stream';
import { join } from 'node:path';

/** Adresse interne de l'API Rust (Axum). */
const API_URL = (process.env['API_URL'] ?? 'http://127.0.0.1:8080').replace(/\/$/, '');
const PORT = Number(process.env['PORT'] || 4000);

const browserDistFolder = join(import.meta.dirname, '../browser');

const app = express();
app.disable('x-powered-by');
const angularApp = new AngularNodeAppEngine();

/**
 * Proxy vers l'API : le navigateur ne parle qu'à une seule origine, ce qui permet
 * un cookie de session HttpOnly/SameSite sans configuration CORS.
 */
async function proxy(req: Request, res: Response): Promise<void> {
  try {
    const headers = new Headers();
    for (const [key, value] of Object.entries(req.headers)) {
      if (value === undefined || ['host', 'connection', 'content-length'].includes(key)) continue;
      headers.set(key, Array.isArray(value) ? value.join(', ') : value);
    }
    headers.set('x-forwarded-for', req.ip ?? '');
    headers.set('x-forwarded-proto', req.protocol);

    const hasBody = !['GET', 'HEAD'].includes(req.method);
    const upstream = await fetch(API_URL + req.originalUrl, {
      method: req.method,
      headers,
      body: hasBody ? (Readable.toWeb(req) as ReadableStream) : undefined,
      // @ts-expect-error option Node.js requise pour un corps en flux
      duplex: hasBody ? 'half' : undefined,
      redirect: 'manual',
    });

    res.status(upstream.status);
    upstream.headers.forEach((value, key) => {
      if (!['content-encoding', 'content-length', 'transfer-encoding', 'connection'].includes(key)) {
        res.setHeader(key, value);
      }
    });
    const cookies = upstream.headers.getSetCookie();
    if (cookies.length) res.setHeader('set-cookie', cookies);

    if (upstream.body) {
      Readable.fromWeb(upstream.body as never).pipe(res);
    } else {
      res.end();
    }
  } catch (err) {
    console.error('Erreur du proxy API', err);
    res.status(502).json({ error: 'API indisponible' });
  }
}

app.use('/api', proxy);
app.get(['/sitemap.xml', '/rss.xml'], proxy);

/** Fichiers statiques du build (noms hashés : cache long). */
app.use(
  express.static(browserDistFolder, {
    maxAge: '1y',
    index: false,
    redirect: false,
    setHeaders: (res, path) => {
      // Les fichiers non hashés (images, favicon, robots.txt) sont revalidés plus souvent.
      if (!/\.[0-9a-z]{8,}\.(js|css|woff2?)$/i.test(path) && !/-[A-Z0-9]{8}\.(js|css)$/.test(path)) {
        res.setHeader('Cache-Control', 'public, max-age=86400');
      }
    },
  }),
);

/** Toutes les autres requêtes : rendu Angular (SSR ou shell client selon la route). */
app.use((req, res, next) => {
  // Pendant le rendu serveur, Angular résout les URL relatives (`/api/...`) à partir de
  // l'hôte de la requête. On le remplace par l'adresse locale : les appels à l'API restent
  // internes au serveur au lieu de repasser par le domaine public.
  req.headers.host = `127.0.0.1:${PORT}`;
  delete req.headers['x-forwarded-host'];
  delete req.headers['x-forwarded-proto'];
  delete req.headers['x-forwarded-port'];
  angularApp
    .handle(req)
    .then((response) => (response ? writeResponseToNodeResponse(response, res) : next()))
    .catch(next);
});

if (isMainModule(import.meta.url) || process.env['pm_id']) {
  app.listen(PORT, (error) => {
    if (error) {
      throw error;
    }
    console.log(`Serveur SSR à l'écoute sur http://localhost:${PORT} (API : ${API_URL})`);
  });
}

export const reqHandler = createNodeRequestHandler(app);
