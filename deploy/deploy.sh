#!/usr/bin/env bash
# Déploie un environnement GoodPlace sur ce serveur : deploy/deploy.sh prod|preprod|test
# Chaque environnement a son projet compose (conteneurs et base séparés), son port local
# et son fichier d'environnement, rangé hors du dépôt dans $GOODPLACE_ENV_DIR (~/goodplace-env).
set -euo pipefail

env_name="${1:?usage : deploy.sh prod|preprod|test}"
case "$env_name" in
  prod)    project=goodplace;         port=4000 ;;
  preprod) project=goodplace-preprod; port=4001 ;;
  test)    project=goodplace-test;    port=4002 ;;
  *) echo "Environnement inconnu : $env_name" >&2; exit 1 ;;
esac

root="$(cd "$(dirname "$0")/.." && pwd)"
env_file="${GOODPLACE_ENV_DIR:-$HOME/goodplace-env}/$env_name.env"
[[ -f "$env_file" ]] || { echo "Fichier d'environnement manquant : $env_file" >&2; exit 1; }

export GOODPLACE_ENV_FILE="$env_file" WEB_PORT="$port"
compose=(docker compose -p "$project" -f "$root/docker-compose.yml" --env-file "$env_file")

echo "==> Construction et démarrage de $project (127.0.0.1:$port)"
"${compose[@]}" up -d --build --remove-orphans

echo "==> Vérification de santé"
for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:$port/api/health" >/dev/null 2>&1; then
    echo "OK : $env_name répond sur 127.0.0.1:$port"
    docker image prune -f >/dev/null
    exit 0
  fi
  sleep 2
done

echo "ÉCHEC : $env_name ne répond pas après 2 minutes" >&2
"${compose[@]}" ps >&2
"${compose[@]}" logs --tail 50 api web >&2
exit 1
