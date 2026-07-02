cd edgequake/docker
POSTGRES_PORT=5433 FRONTEND_PORT=3003 docker compose --env-file ../.env up -d

cd /tmp/edgequake/edgequake && SQLX_OFFLINE=true cargo check -p edgequake-api 2>&1 | tail -5
cd /tmp/edgequake/edgequake/docker && docker compose --env-file ../.env build edgequake frontend 2>&1 | tail -5 && docker compose --env-file ../.env up -d edgequake frontend 2>&1 | tail -3
