# SPEC-183 — Neo4j en option : projection graphe en lecture, à côté de PostgreSQL

Statut : **proposition, rien d'implémenté**. Base : v0.39.1 (branche `feat/agents-x-copilot-v0.39.1`).

## 1. Réponse courte

**Oui, c'est possible, à une condition : Neo4j ne remplace aucune brique existante.**
Il s'ajoute, en option, comme une **copie en lecture** du graphe. PostgreSQL reste la source de vérité et le chemin de service de toutes les fonctions actuelles.

| Option | Perf actuelle | Régressions | Verdict |
|---|---|---|---|
| A. Remplacer le stockage graphe PostgreSQL par Neo4j (source de vérité) | Perdue : chaque requête RAG fait plusieurs appels de graphe groupés ; chacun deviendrait un aller-retour réseau HTTP au lieu d'une requête SQL locale | Élevées : le commit atomique document + graphe + vecteurs dans une seule transaction PostgreSQL est cassé (deux bases, pas de transaction commune) ; l'isolation tenant par SQL/RLS doit être refaite | **Rejetée** |
| B. Neo4j en **projection asynchrone en lecture**, alimentée depuis PostgreSQL, et qui ne sert que des **capacités nouvelles** | Inchangée : l'ingestion ne l'attend pas et les requêtes existantes ne la touchent pas | Aucune si le flag est éteint (défaut) ; si Neo4j tombe, seules les capacités nouvelles sont indisponibles | **Retenue** |

« Meilleur que Neo4j managé sur GCP » se comprend ainsi, honnêtement : EdgeQuake + Neo4j **n'est pas une meilleure base graphe que Neo4j** ; c'est la même base. La différence est tout ce qu'Aura ne fait pas seul et qu'EdgeQuake apporte au-dessus :

- l'extraction d'entités et de relations depuis les documents (pipeline LLM) ;
- la recherche hybride vecteurs + graphe (modes RAG) ;
- le multi-tenant (organisations, espaces de travail, rôles, SSO, audit) ;
- l'interface, l'API, MCP et le copilote.

La cible Neo4j de cette spec peut d'ailleurs **être** une instance Aura sur GCP : on ne choisit pas l'un contre l'autre.

## 2. Ce qui existe déjà (SPEC-149, prototype)

Dans `edgequake/crates/edgequake-storage/src/adapters/neo4j/` (feature Cargo `neo4j`, **éteinte par défaut**) :

| Élément | Fichier | État |
|---|---|---|
| Client HTTP Query API v2, transactions, configuration par env | `client.rs` | prototype |
| Création du schéma (contraintes, index) | `provision.rs` | prototype |
| Upsert/suppression de révisions d'entités et d'arêtes, `GraphStorageMutateOps` | `mutate.rs` | prototype |
| Lecture scopée tenant/workspace, traversée bornée, digests de révision | `read.rs` | prototype |
| Rejeu PostgreSQL → Neo4j par pages, comparaison de digests | `migration.rs` | prototype |
| Sélection du profil `P2a` / `P2b` et vérification de connectivité | `edgequake-api/src/state/data_access_factory.rs` | **bloquée** : `assert_product_serving_allowed` refuse tout profil autre que P0 |
| File de projection (claim/renew/ack, fenced) | `edgequake-storage/src/projection/ledger.rs` | en service pour les projections existantes |

Deux limites du prototype :

1. **Layout « arête = nœud »** : `(:EdgeRevision)-[:FROM]->(:EntityKey)` et `-[:TO]->`. Un saut logique coûte deux sauts Neo4j. Le Cypher naturel `(a)-[r]->(b)` est impossible et les projections GDS doivent passer par une projection Cypher.
2. Il vise le **remplacement** (profil P2a : Neo4j sert le graphe) et non une projection additive.

Cette spec réutilise le client, le provisioning, le rejeu et les digests, et change le layout et le rôle.

## 3. Ce que Neo4j apporterait en plus (et seulement ça)

Ce sont les points où le benchmark local (1 M nœuds, 2,98 M arêtes, `docs/concepts/graph-storage-at-scale-fr.md`) montre une limite de PostgreSQL, ou qu'EdgeQuake ne propose pas :

| Capacité | Aujourd'hui (PostgreSQL) | Avec la projection |
|---|---|---|
| Cypher en lecture | Absent depuis le retrait d'AGE | Endpoint Cypher en lecture seule, scopé (§5) |
| Traversées profondes (> 3 sauts), plus court chemin | BFS plafonné à 3 sauts / 500 nœuds ; plus court chemin 4 sauts ≈ 8 s ; 2 sauts non plafonnés depuis un hub ≈ 6,7 s | Exécutées dans Neo4j, objectif à mesurer (§7) |
| Algorithmes de graphe (communautés, PageRank, centralité, similarité) | Absents | Neo4j GDS sur la projection ; résultats **réécrits dans PostgreSQL** en propriétés de nœuds |
| Exploration visuelle avancée | Vue graphe EdgeQuake | Neo4j Browser / Bloom en lecture, en plus |

Ce qui **ne change pas** : l'ingestion, l'extraction, les modes RAG, le chat, le copilote, la vue graphe actuelle, les recherches de labels, les compteurs. Tout reste servi par PostgreSQL.

## 4. Architecture

```
          écriture (inchangée)                       lecture (inchangée)
docs ──► pipeline ──► commit PostgreSQL ◄──────────── RAG / UI / API / MCP / copilote
                        │ (graphe + vecteurs + ledger de projection, 1 transaction)
                        ▼
              worker de projection (nouveau consommateur du ledger existant)
                        │ async, idempotent, par révision
                        ▼
                 Neo4j (Community, Enterprise ou Aura)
                        │
     capacités nouvelles uniquement : /graph/cypher, /graph/paths, /graph/analytics
                        │
     résultats GDS ──► réécrits dans PostgreSQL (propriétés), donc visibles partout
```

Règles :

- **R1. PostgreSQL est l'unique autorité.** Neo4j peut être effacé et reconstruit à tout moment par rejeu (`migration.rs`).
- **R2. Aucun chemin existant n'appelle Neo4j.** Seuls les nouveaux endpoints le font. Le profil de service reste P0 ; on ajoute un profil **`P0+N`** = P0 plus une projection Neo4j déclarée. Le gate fail-closed est étendu pour accepter ce profil, sans assouplir P1/P2.
- **R3. Écritures asynchrones.** Le commit d'ingestion écrit une ligne dans le ledger (mécanisme existant) ; le worker la livre à Neo4j. Neo4j lent ou absent ⇒ la file grossit, l'ingestion ne ralentit pas.
- **R4. Fraîcheur exposée.** Chaque réponse des nouveaux endpoints indique `projection_lag` (révision PostgreSQL vs révision Neo4j du workspace). Au-delà d'un seuil configurable, l'endpoint répond `503 projection_stale` au lieu de servir des données fausses.
- **R5. GDS ne s'écrit jamais directement dans Neo4j comme vérité.** Les scores (communauté, PageRank…) sont réécrits dans PostgreSQL par un job, avec horodatage et version d'algorithme, puis re-projetés comme le reste.

### Layout de la projection (nouveau)

```
(:Entity {tenant_id, workspace_id, name, entity_type, description, physical_id, revision, digest})
  -[:RELATED {relationship_type, weight, keywords, physical_id, revision, digest}]->
(:Entity)
```

- Clés alignées sur la migration 170 : nœud unique par `(namespace, node_id)` comme `eq_graph_nodes` ; une relation `:RELATED` par `(namespace, source_id, target_id, rel_type)` comme `eq_graph_edges`, donc plusieurs relations de types différents possibles entre deux mêmes entités. `tenant_id` et `workspace_id` sont des propriétés indexées.
- On garde `physical_id`, `revision` et `digest` pour conserver le rejeu idempotent et la comparaison de digests du prototype.
- Le type de relation reste une **propriété** et non un type Neo4j dynamique, pour borner le schéma et rester compatible avec GDS.
- Un label supplémentaire par workspace (`:WS_<hash>`) accélère les projections GDS par workspace. Ce n'est **pas** un mécanisme de sécurité (§5).

## 5. Isolation des tenants (point dur)

Un Cypher libre peut contourner un filtre `WHERE workspace_id = …`. Deux niveaux :

| Déploiement Neo4j | Ce qu'on expose |
|---|---|
| **Community** (une base par instance) | Pas de Cypher libre. Seulement un **catalogue de requêtes paramétrées** côté serveur (chemins, voisinage profond, k-hop, algorithmes), où le scope est injecté par EdgeQuake et jamais par l'utilisateur. |
| **Enterprise / Aura** (plusieurs bases) | Option : **une base Neo4j par workspace**, et le Cypher libre en **transaction de lecture** sur la seule base du workspace de l'appelant, avec un utilisateur Neo4j en lecture seule, un timeout et une limite de lignes. |

Dans les deux cas :

- le contrôle d'accès EdgeQuake (rôle workspace) passe avant l'appel ;
- chaque requête est auditée (`edgequake-audit`) ;
- le copilote n'obtient que le catalogue paramétré, jamais le Cypher libre.

## 6. Garanties « zéro régression »

| # | Garantie | Comment on la vérifie |
|---|---|---|
| G1 | Build par défaut identique | Feature `neo4j` éteinte par défaut ; CI : `cargo build` / `clippy` sans la feature, binaire et tests inchangés |
| G2 | Runtime par défaut identique | `EDGEQUAKE_GRAPH_PROJECTION` absent ⇒ aucun worker, aucun endpoint, aucune connexion. Test : démarrage P0 avec et sans la variable, mêmes routes hors nouveaux endpoints |
| G3 | Toute la suite actuelle passe en `P0+N` | La CI rejoue les suites core/pipeline/API/e2e DB **et** web UI avec Neo4j actif : mêmes résultats qu'en P0 |
| G4 | Ingestion non ralentie | Benchmark d'ingestion (même corpus) en P0 et en P0+N : écart du p95 ≤ 2 % ; Neo4j arrêté pendant l'ingestion ⇒ l'ingestion termine quand même |
| G5 | Requêtes RAG non touchées | Test qui échoue si un handler existant appelle le client Neo4j (le client n'est injecté que dans les nouveaux handlers) |
| G6 | Cohérence vérifiable | Job de digests PostgreSQL vs Neo4j par workspace (`compare_digest_page` existant) ; 0 écart après rattrapage de la file |
| G7 | Suppression propre | Suppression ou purge d'un workspace/tenant ⇒ suppression dans Neo4j via le ledger ; test : 0 nœud restant pour ce scope |
| G8 | Retour arrière | Retirer la variable ⇒ retour exact à P0 ; la base Neo4j peut être jetée sans perte |

## 7. Objectifs de performance (à mesurer, pas promis)

Sur la même machine et le même jeu que le benchmark PostgreSQL (1 M nœuds, 2,98 M arêtes, distribution en loi de puissance), Neo4j en conteneur local :

| Mesure | PostgreSQL (mesuré) | Objectif Neo4j |
|---|---|---|
| Plus court chemin 4 sauts | ≈ 8 s | < 1 s |
| 2 sauts non plafonnés depuis le hub principal | ≈ 6,7 s | < 2 s |
| 3 à 6 sauts plafonnés | 3 sauts plafonnés ≈ 48 ms | < 200 ms jusqu'à 6 sauts |
| Retard de projection p95 en ingestion continue | — | < 5 s |
| Reconstruction complète 1 M nœuds par rejeu | — | < 30 min |
| PageRank / Louvain sur 1 M nœuds (GDS) | — | à mesurer, réécriture PG incluse |

Si un objectif n'est pas atteint, la capacité correspondante n'est pas livrée et PostgreSQL reste la seule réponse. Les mesures sont rangées dans `measurements/` comme dans SPEC-164-pure-postgres-study.

## 8. Lots de travail

| Lot | Contenu | Dépend de |
|---|---|---|
| L0 | Benchmark Neo4j local sur le jeu 1 M (`graph-bench`), décision go/no-go par capacité | — |
| L1 | Layout natif `(:Entity)-[:RELATED]->(:Entity)` dans `mapping.rs`/`mutate.rs`/`provision.rs`, digests conservés | L0 |
| L2 | Worker de projection : consommateur du ledger pour le provider `neo4j`, retries, métriques (lag, file, erreurs) | L1 |
| L3 | Profil `P0+N` dans `data_access_factory.rs` ; gate étendu ; readiness non bloquante (Neo4j KO ⇒ `/ready` OK, capacité marquée indisponible) | L2 |
| L4 | Endpoints `/graph/paths` et `/graph/neighborhood?depth≤6` (catalogue paramétré), contrôle d'accès, audit, `projection_lag` | L3 |
| L5 | Job GDS (communautés, PageRank) → réécriture PostgreSQL → affichage dans la vue graphe existante | L3 |
| L6 | Cypher libre en lecture, Enterprise/Aura uniquement, une base par workspace | L4 + choix d'édition |
| L7 | Outil copilote `graph_paths` (catalogue seulement), libellés en/fr/zh | L4 |
| L8 | CI : matrice P0 et P0+N, tests G1–G8, doc opérateur (Compose, Helm, Aura) | L2–L7 |

## 9. Définition de « terminé »

- G1–G8 vérifiés et automatisés en CI.
- Objectifs §7 mesurés et publiés, y compris ceux qui ne sont pas atteints.
- Doc opérateur : activation, dimensionnement, reconstruction, retour arrière.
- Aucune modification de comportement en P0, prouvée par la suite complète inchangée.

## 10. Risques et questions ouvertes

| Sujet | Détail | À trancher par |
|---|---|---|
| Licences | Neo4j Community est sous GPLv3, en processus séparé appelé par HTTP. Enterprise, Aura et les fonctions GDS avancées sont commerciales. GDS Community a des limites (notamment la parallélisation). **À vérifier sur les conditions Neo4j en vigueur avant toute offre.** | Juridique / client |
| Coût d'exploitation | Une base de plus à opérer : sauvegarde (reconstructible par rejeu), supervision, mise à jour | Ops |
| Données obsolètes | Les capacités nouvelles peuvent servir un graphe en retard de quelques secondes ; R4 refuse au-delà du seuil | Produit |
| Upstream | Le mainteneur a choisi « PostgreSQL seul » (SPEC-164-pure-postgres-study). Cette spec reste additive et derrière un flag pour pouvoir vivre dans le fork sans conflit, ou être proposée en amont | Mainteneur |
| SQL/PGQ | Si PostgreSQL 20 livre SQL/PGQ, une partie de L4 (chemins) pourrait revenir dans PostgreSQL ; L0 sert aussi de point de comparaison | Revue à la sortie de PG20 |
| Échelle multi-tenant | Une base par workspace (L6) ne passe pas à des milliers de workspaces sur une seule instance ; au-delà, catalogue paramétré seulement | Mesure L6 |
