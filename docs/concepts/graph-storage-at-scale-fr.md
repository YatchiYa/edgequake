# EdgeQuake : le graphe dans PostgreSQL, ses limites, Cypher et l'échelle du million

*Réponses fondées sur le code d'EdgeQuake v0.39.1, sur l'étude de l'auteur (`specs/164-pure-postgres-study`) et sur un banc d'essai indépendant à 1 million d'entités (novembre 2026). Les chiffres sont des mesures, pas des estimations ; leurs limites sont indiquées à la fin.*

## Summary (English)

**Graph storage at scale: PostgreSQL limits, Cypher after Apache AGE, and Neo4j.** Since v0.35 the knowledge graph is stored in two plain PostgreSQL tables (`eq_graph_nodes`, `eq_graph_edges`); Apache AGE is optional and no longer used. EdgeQuake never ran Cypher on the query path: it runs batched, indexed SQL, one query per hop, capped at 3 hops and 500 nodes. Measured on 1 million entities and 3 million relationships with realistic hubs: entity lookup 0.08 ms, capped 2-hop neighbourhood 15 ms, capped 3-hop 48 ms, label search 99 ms. What PostgreSQL does poorly: uncapped deep traversal (6.7 s for 428,000 nodes), shortest paths (8 s for 4 hops) and graph algorithms (none built in). Neo4j becomes relevant for those needs (paths, algorithms such as communities or centrality, analysts exploring in Cypher), not for volume: 1 million nodes is small for both engines; add it as a read-only projection rather than replacing PostgreSQL. Cypher was lost with AGE for people querying the database; plain SQL and recursive CTEs cover it, and the ISO SQL/PGQ syntax (GRAPH_TABLE MATCH) was added to PostgreSQL 19 then withdrawn in September 2026, now expected in PostgreSQL 20. At 1 million nodes the real costs are LLM extraction during ingestion and about 8 GB of embeddings whose HNSW indexes must fit in memory, not the graph (1.1 GB).

## En une phrase

PostgreSQL tient très bien 1 million d'entités **pour ce que fait EdgeQuake** (voisinage local plafonné, en quelques millisecondes) ; il devient coûteux pour les **parcours profonds non bornés, la recherche de chemins et les algorithmes de graphe**, et c'est précisément là — pas sur le volume — que Neo4j redevient pertinent.

## Comment EdgeQuake stocke et lit le graphe aujourd'hui

- Deux tables ordinaires : `eq_graph_nodes` (une ligne par entité, avec son degré pré-calculé) et `eq_graph_edges` (une ligne par relation), indexées sur la source, la cible, l'espace de travail, le texte (trigrammes) et la provenance (GIN). Migration `170_spec164_plain_graph.sql`.
- Le parcours se fait en **largeur, saut par saut** : une requête indexée par saut sur toute la « frontière », plafonnée à **3 sauts et 500 nœuds** (`graph_traversal.rs`). Le coût dépend donc de la taille du voisinage visité, **pas de la taille du graphe**.
- EdgeQuake **n'utilisait déjà pas Cypher** sur le chemin des requêtes avant la suppression d'AGE : l'étude de l'auteur recense uniquement du SQL par lots (lecture d'un nœud, arêtes incidentes, expansion k-sauts, degrés, recherche par libellé). AGE servait de format de stockage, avec un surcoût (`agtype`).

## 1. Quelles limites ? Neo4j devient-il pertinent à certaines échelles ?

### Ce que montrent les mesures

**Étude de l'auteur** (graphe synthétique régulier, PostgreSQL 18.6, AGE 1.8) — tables simples comparées à AGE :

| Échelle | Nœuds | Arêtes | Opérations du produit | Tables simples vs AGE |
|---|---:|---:|---|---|
| S | 50 000 | 70 000 | 0,4 à 2 ms | équivalent ou plus rapide |
| M | 200 000 | 500 000 | 0,4 à 3 ms | pire rapport 1,09 ; écritures 3,5× plus rapides |
| L | 1 000 000 | 2 000 000 | 0,4 à 3 ms | idem ; écritures 5,5× plus rapides |

Limite reconnue par l'auteur lui-même : graphe synthétique « en anneau », **sans vrais nœuds-pivots** (le cas réel : une entité comme ALTAROC reliée à des centaines d'autres).

**Banc d'essai complémentaire** (pour combler ce manque) — 1 000 000 d'entités, 2 977 801 relations, distribution en **loi de puissance** comme dans un vrai corpus : l'entité la plus connectée a 139 677 liens, la médiane 4, le 99e centile 25. Tables et index identiques à la migration 170, PostgreSQL 18, une seule session.

| Requête | Médiane | Pire cas |
|---|---:|---:|
| Lire une entité | 0,08 ms | 2,5 ms |
| Liens d'une entité ordinaire | 0,10 ms | 0,15 ms |
| Liens d'un grand pivot, plafonnés à 500 (produit) | 2,8 ms | 6 ms |
| Voisinage 2 sauts, plafond 500 (produit) | 15 ms | 204 ms |
| Voisinage 3 sauts, plafond 500 (produit) | 48 ms | 383 ms |
| 20 entités les plus connectées (accueil du graphe) | 0,09 ms | 0,16 ms |
| Recherche par fragment de nom (trigrammes) | 99 ms | 154 ms |
| **2 sauts sans plafond** depuis le plus gros pivot (428 000 nœuds atteints) | **6,7 s** | — |
| **2 sauts sans plafond** depuis une entité ordinaire (141 000 nœuds atteints) | **0,19 s** | — |
| **Plus court chemin ≤ 4 sauts** (CTE récursive) | **8 s** | 10 s |

Chargement complet du million : environ 1 minute ; taille des deux tables avec index : 1,1 Go.

### Lecture

- **Ce qui ne pose aucun problème** : tout ce que le produit fait, parce qu'il plafonne. Les plafonds ne sont pas une faiblesse : à 2 sauts, même une entité banale touche 14 % du graphe dès qu'un de ses voisins est un pivot. Aucun moteur ne rend utile une réponse construite sur 141 000 nœuds.
- **Où PostgreSQL paie** : chaque saut est une jointure. Sans plafond, le coût suit le nombre de nœuds touchés (6,7 s pour 428 000). Pour un **plus court chemin**, une CTE récursive explore toutes les branches, d'où 8 s. Et PostgreSQL n'a **pas de bibliothèque d'algorithmes de graphe** (PageRank, communautés, centralité).
- **Ce qu'un moteur natif comme Neo4j fait mieux** : il suit des pointeurs au lieu de joindre des index (*index-free adjacency*), et il fournit Cypher, la recherche de chemins et la bibliothèque GDS. Pour des parcours profonds ou des chemins, l'écart est typiquement d'un ou deux ordres de grandeur. Pour le voisinage plafonné du produit, il n'y a rien à gagner : on est déjà à quelques millisecondes.

### Quand Neo4j devient pertinent

Ce n'est **pas une question de volume** : un million d'entités est petit pour les deux moteurs. Neo4j se justifie quand le **besoin fonctionnel** change :

1. des questions qui exigent des **chemins** (« comment A est-il relié à B ? », chaînes d'influence, de propriété, de causalité) ;
2. des **algorithmes de graphe** comme fonctionnalité (détection de communautés, centralité, similarité) ;
3. des **analystes** qui explorent le graphe en Cypher, interactivement et en profondeur.

Le prix : une **deuxième base** à synchroniser, sauvegarder et sécuriser, et surtout la perte de l'isolation par organisation de PostgreSQL (RLS), qu'il faut reconstruire. À noter : la v0.39.1 contient déjà un adaptateur Neo4j (SPEC-149), conçu comme une **projection** du graphe PostgreSQL, mais il est marqué « non disponible en production » (seul le profil P0, PostgreSQL, est servi).

**Recommandation** : rester sur PostgreSQL comme source de vérité. Si un besoin de chemins ou d'algorithmes apparaît, ajouter une projection en lecture seule (Neo4j, ou Memgraph) alimentée depuis PostgreSQL, plutôt que de déplacer le stockage.

## 2. A-t-on perdu Cypher ? Comment interroger le graphe avec élégance et performance ?

**Oui, Cypher a disparu avec AGE**, mais seulement pour un humain ou un outil externe qui voudrait interroger la base : le produit, lui, ne s'en servait pas. Pour interroger aujourd'hui :

**SQL direct sur deux tables**, souvent plus simple qu'on ne le croit (les identifiants sont préfixés par l'espace de travail : `<workspace_id>::ALTAROC`) :

```sql
-- Voisins d'une entité avec le type de relation
SELECT e.rel_type, n.node_id, n.entity_type
FROM eq_graph_edges e
JOIN eq_graph_nodes n ON n.namespace = e.namespace AND n.node_id = e.target_id
WHERE e.namespace = $1 AND e.source_id = $2;   -- $2 = '<workspace_id>::ALTAROC'

-- Équivalent Cypher : MATCH (a)-[:INVESTS_IN]->(f)<-[:INVESTS_IN]-(co) RETURN co   (co-investisseurs)
SELECT DISTINCT p.source_id
FROM eq_graph_edges a
JOIN eq_graph_edges p ON p.namespace = a.namespace AND p.target_id = a.target_id
WHERE a.namespace = $1 AND a.source_id = $2
  AND a.rel_type = 'INVESTS_IN' AND p.rel_type = 'INVESTS_IN' AND p.source_id <> $2;
```

Ces requêtes ont été exécutées sur une base EdgeQuake réelle. Une réserve : l'extracteur écrit surtout le type générique `RELATED_TO` (dans une base de test : 1 630 liens sur environ 1 750, contre 28 `INVESTS_IN` et 59 `PART_OF`), la nature du lien restant en texte dans les propriétés. Les requêtes « par type de relation » ne valent donc que si l'extraction produit des types précis. C'est une limite de la modélisation, pas du moteur : elle vaudrait aussi pour Cypher.

**Requêtes récursives**, pour une profondeur variable. Depuis PostgreSQL 14, `SEARCH BREADTH FIRST` et `CYCLE` rendent les parcours lisibles et sans boucle infinie ; il faut toujours borner la profondeur :

```sql
WITH RECURSIVE voisinage(node, depth) AS (
  SELECT $2::text, 0
  UNION
  SELECT e.target_id, v.depth + 1
  FROM voisinage v
  JOIN eq_graph_edges e ON e.namespace = $1 AND e.source_id = v.node
  WHERE v.depth < 3
) CYCLE node SET est_cycle USING chemin
SELECT node, min(depth) FROM voisinage GROUP BY node ORDER BY 2 LIMIT 500;
```

**À l'horizon : SQL/PGQ**, la norme ISO (2023) qui apporte une syntaxe proche de Cypher (`GRAPH_TABLE ... MATCH (a)-[r]->(b)`) **directement sur les tables existantes**, donc sur celles d'EdgeQuake sans les modifier. Elle a été intégrée à PostgreSQL 19 en mars 2026, puis **retirée en septembre 2026** avant la sortie, pour être corrigée sans contrainte de compatibilité : attendue désormais pour PostgreSQL 20 (fin 2027, sans engagement). C'est la piste la plus « élégante », mais pas avant cette date.

**Pour les analystes dès maintenant** : exporter le graphe (deux tables, quelques minutes à 1 M d'entités) vers Neo4j ou NetworkX, ou ouvrir l'adaptateur Neo4j en projection lecture seule.

## 3. Sait-on passer de 50 000 à 1 million de nœuds ?

**Côté graphe, oui, c'est mesuré** : les requêtes du produit restent entre 0,1 et 50 ms à 1 million d'entités, même avec des pivots réalistes (tableau ci-dessus). Le graphe lui-même n'est pas le goulot.

Les vrais sujets à 1 M sont ailleurs :

1. **L'ingestion.** Chaque entité vient d'appels au LLM d'extraction : un million d'entités représente des centaines de milliers de passages traités par le modèle. Le temps et le coût d'API dominent tout le reste ; c'est le premier poste à chiffrer, sur un échantillon du vrai corpus.
2. **Les vecteurs.** Chaque entité et chaque relation a un vecteur (1024 dimensions en Mistral, ~2 Ko). Pour 1 M d'entités et ~3 M de relations : de l'ordre de **8 Go de vecteurs**, plus les index HNSW. Il faut que ces index tiennent en mémoire pour rester rapides : c'est ce qui dimensionne le serveur (RAM), bien avant le graphe (1,1 Go).
3. **La qualité.** À cette échelle, la déduplication des entités (« J. Doe » et « Jane Doe ») et la gestion des pivots pèsent davantage sur la pertinence des réponses que la vitesse brute.

**Avant d'y aller** : un essai de charge sur le vrai corpus (pas un graphe synthétique), avec plusieurs utilisateurs simultanés, l'isolation par organisation active, et une mesure de la mémoire nécessaire aux index de vecteurs.

## Limites de ces mesures

- Graphe synthétique : loi de puissance réaliste dans sa forme, pas un vrai corpus.
- Une seule session, sans concurrence, sans la politique d'isolation (RLS) ni le passage par l'API : les temps réels du produit sont un peu plus élevés.
- Machine de développement, PostgreSQL 18 réglé simplement (1 Go de cache).
- La comparaison avec Neo4j est qualitative : aucun Neo4j n'a été mesuré ici.

Scripts reproductibles : `specs/164-copilot-auth-gcp-agents/graph-bench/bench.sql` (construction) et `timings.sql` (mesures).

## Sources

- Étude de l'auteur : `specs/164-pure-postgres-study/reports/PROOF.md` et `WHY-AGE-IS-OPTIONAL.md` (EdgeQuake v0.39.1).
- Adaptateur Neo4j : `specs/149-data-access-improvements/README.md`.
- SQL/PGQ dans PostgreSQL 19, puis son retrait : [Neon — PostgreSQL 19 SQL/PGQ](https://neon.com/postgresql/postgresql-19/sql-pgq-graph-queries), [depesz — Waiting for PostgreSQL 19](https://www.depesz.com/2026/07/31/waiting-for-postgresql-19-sql-property-graph-queries-sql-pgq/), [pgsql-committers — Revert SQL/PGQ](https://postgrespro.com/list/id/E1x3f62-00000003sxD-0vdk@gemulon.postgresql.org), [The Build — SQL/PGQ in PostgreSQL 19](https://thebuild.com/blog/sqlpgq-in-postgresql-19-graph-queries-without-the-graph-database/).
