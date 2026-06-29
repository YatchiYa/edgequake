cd edgequake/docker
POSTGRES_PORT=5433 FRONTEND_PORT=3003 docker compose --env-file ../.env up -d
