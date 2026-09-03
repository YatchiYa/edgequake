# Langfuse — traces absentes : diagnostic et actions

**Version Langfuse concernée :** 3.155.1 OSS · **Déploiement :** Kubernetes, pods séparés

---

## Résumé

Votre instance Langfuse **3.155.1 expose l'endpoint OTLP** utilisé par EdgeQuake. La
configuration actuelle force pourtant le transport `ingestion`, qui est un mode de
repli conçu pour les versions **antérieures à la 3.22**.

**Action principale : repasser le transport en `auto`.** Aucune mise à jour
d'EdgeQuake ni de Langfuse n'est nécessaire.

---

## 1. Vérification effectuée

Une instance Langfuse **3.155.1** a été déployée et interrogée sur l'endpoint exact
qu'utilise EdgeQuake :

| Requête | Résultat |
|---|---|
| `POST /api/public/otel/v1/traces` sans authentification | **401** |
| `POST /api/public/otel/v1/traces` avec authentification | **200** |
| `POST /api/public/ingestion` avec authentification | **207** |

L'endpoint OTLP est donc **présent et fonctionnel** sur votre version. Les deux
transports sont utilisables ; OTLP est le chemin nominal et le mieux éprouvé.

> À noter : la recommandation initiale de forcer `ingestion` visait Langfuse **3.1.x**,
> où l'endpoint OTLP renvoie effectivement 404. Elle ne s'applique pas à la 3.155.

---

## 2. Action recommandée

### 2.1 Repasser le transport en détection automatique

```yaml
# Deployment EdgeQuake
- name: EDGEQUAKE_LANGFUSE_API
  value: "auto"        # ou supprimer entièrement la variable
```

En mode `auto`, EdgeQuake teste l'endpoint OTLP au démarrage et retient le transport
adapté. Sur votre version, il choisira **OTLP**.

### 2.2 Ne conserver qu'une seule variable d'URL

`LANGFUSE_BASE_URL` et `LANGFUSE_HOST` désignent la même cible ; la seconde n'est
qu'un repli. En conserver deux expose à une divergence silencieuse.

```yaml
- name: LANGFUSE_BASE_URL
  value: "https://langfuse-edgequake.ppd.datascience.analytics.safran" ou autre ...
# LANGFUSE_HOST : à supprimer
```

**Contraintes sur cette valeur :**

| Règle | Raison |
|---|---|
| Aucun chemin, aucun `/` final | EdgeQuake ajoute lui-même le chemin du transport |
| Jamais vide | Une chaîne vide équivaut à « non définie » et déclenche un repli vers `cloud.langfuse.com` |
| Jamais `localhost` | Dans un pod, `localhost` désigne le pod lui-même |

### 2.3 Redémarrer et vérifier le transport retenu

```bash
kubectl rollout restart deploy/edgequake -n <namespace>
kubectl logs -n <namespace> deploy/edgequake | grep -i "langfuse"
```

Attendu : une ligne indiquant le transport sélectionné et l'endpoint utilisé.

---

## 3. Si les traces manquent toujours

### 3.1 Rendre les erreurs visibles

Les échecs d'export sont journalisés en `DEBUG`. Avec `RUST_LOG=info` (défaut en
production), **un export qui échoue est totalement silencieux**. Le temps du
diagnostic :

```bash
kubectl set env deploy/edgequake -n <namespace> \
  RUST_LOG=info,opentelemetry_sdk=debug,opentelemetry_otlp=debug

kubectl logs -n <namespace> deploy/edgequake --tail=200 \
  | grep -iE 'langfuse|otlp|certificate|tls|401|403'
```

### 3.2 Contrôles, dans l'ordre

Ne pas passer au suivant tant que le précédent échoue.

| # | Contrôle | Commande | Attendu |
|---|---|---|---|
| 1 | Variables injectées, non vides | `kubectl exec deploy/edgequake -- env \| grep LANGFUSE` | 3 valeurs renseignées |
| 2 | Cible réellement utilisée | `curl -s localhost:8080/api/v1/settings/langfuse \| jq .base_url` | votre URL interne, **pas** `cloud.langfuse.com` |
| 3 | Langfuse joignable depuis le pod | `curl <url>/api/public/health` | `200` |
| 4 | Clés valides | `curl -u "$LANGFUSE_PUBLIC_KEY:$LANGFUSE_SECRET_KEY" <url>/api/public/projects` | le projet attendu |
| 5 | Migrations Langfuse terminées | `kubectl logs deploy/langfuse-worker \| grep -ci "does not exist"` | `0` |
| 6 | Traces réellement stockées | `curl -u pk:sk "<url>/api/public/traces?limit=5"` | liste non vide après une requête RAG |

> **Point important** : `export_active: true` signifie seulement que des clés sont
> configurées. Cet indicateur ne prouve **ni** que l'URL est correcte, **ni** que les
> traces arrivent. Seul le contrôle 6 est probant.

### 3.3 Certificat interne (à vérifier si le mode `ingestion` doit être conservé)

Votre URL est en HTTPS sur un domaine interne, donc servie par un certificat émis par
votre autorité de certification d'entreprise. Selon la manière dont le client HTTP est
compilé, celui-ci peut ne faire confiance qu'aux autorités publiques et **rejeter**
un certificat interne — l'erreur apparaissant alors uniquement en `DEBUG`.

Signature dans les journaux (avec le niveau DEBUG de §3.1) :
```
certificate verify failed · UnknownIssuer · invalid peer certificate
```

Si cette signature apparaît, deux remèdes :

```yaml
# a) désigner explicitement le magasin de certificats du conteneur
- name: SSL_CERT_FILE
  value: /etc/ssl/certs/ca-certificates.crt

# b) monter le bundle de l'autorité interne et le référencer
```

Ce point ne concerne **pas** le transport OTLP recommandé en §2.1, qui s'appuie sur le
magasin de certificats du système.

### 3.4 Course au démarrage côté Langfuse

Le pod `langfuse-worker` peut démarrer avant la fin des migrations appliquées par le
pod `web`. Les requêtes sont alors acceptées (**200**) mais **aucune trace n'est
créée** — mode de défaillance particulièrement trompeur.

```bash
kubectl logs -n <namespace> deploy/langfuse-worker | grep -ci "does not exist"   # attendu : 0
kubectl rollout restart deploy/langfuse-web deploy/langfuse-worker -n <namespace>
```

Prévention durable : un `initContainer` sur le worker attendant
`/api/public/health` du web.

---

## 4. Coûts affichés à 0,00 $

Langfuse calcule les coûts à partir de **son propre catalogue de modèles** ; EdgeQuake
n'émet jamais de montant, par conception, afin que Langfuse reste la source unique de
vérité.

Le catalogue livré avec une version donnée de Langfuse ne contient que les modèles
connus à sa date de publication. Les modèles récents — GPT-5, Gemini 2.5, Claude 4.x,
Mistral — en sont absents et affichent donc **0,00 $** malgré des jetons correctement
remontés.

**Correctif** : alimenter le catalogue une fois, via l'API publique de Langfuse
(`POST /api/public/models`), avec les tarifs de vos modèles. Un utilitaire de
synchronisation peut vous être fourni sur demande.

**Limite connue** : sans décompte de jetons, aucun coût n'est calculable. C'est le cas
des observations d'embedding, dont l'API ne remonte pas de décompte.

---

## 5. Synthèse

| # | Action  
|---|---|---|
| 1 | `EDGEQUAKE_LANGFUSE_API=auto` (ou supprimer la variable)  
| 2 | Supprimer `LANGFUSE_HOST`, ne garder que `LANGFUSE_BASE_URL`  
| 3 | Redémarrer, vérifier le transport dans les journaux  
| 4 | Si échec : activer le niveau DEBUG et dérouler les 6 contrôles  
| 5 | Coûts : alimenter le catalogue de modèles Langfuse  

