curl 'http://localhost:8049/api/v1/chat/completions/stream' \
  -H 'Accept: */*' \
  -H 'Accept-Language: fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7' \
  -H 'Connection: keep-alive' \
  -H 'Origin: http://localhost:8048' \
  -H 'Referer: http://localhost:8048/' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-site' \
  -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36' \
  -H 'content-type: application/json' \
  -H 'sec-ch-ua: "Chromium";v="146", "Not-A.Brand";v="24", "Google Chrome";v="146"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "Linux"' \
  -H 'traceparent: 00-ee6656d5ae0e333fa5e11b1b49e00846-67429ee5477894c0-01' \
  -H 'x-request-id: 74b431b2-fb0e-4f1b-a740-86bb598e7282' \
  -H 'x-tenant-id: b9d60c6e-c568-487a-a3b9-27318da1549d' \
  -H 'x-user-id: f640c273-9374-4c8f-8ad8-d7f582994a10' \
  -H 'x-workspace-id: 4382e7ab-6fb6-48b6-b49c-c3d1841dcc69' \
  --data-raw $'{"message":"c\'est quoi le produit odyssey 2026 altaroc ?","mode":"global","max_tokens":2048,"temperature":0.7,"top_k":10,"stream":true,"language":"fr"}'


  data: {"type":"conversation","conversation_id":"88d2fc80-5d22-49ee-a557-f3662419dadf","user_message_id":"52284ae9-d1ae-4897-9840-1d14d0935ff2"}

data: {"type":"context","sources":[{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6","score":4.005089,"snippet":"res** :\n  - Société d'investissement existant depuis plus de 15 ans\n  - Historique de performance 15 ans dans le 1er quartile\n  - Taux de perte historique < 2%\n  - Taille du fonds en levée > 1 Md€ et ","reference_id":1,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":337,"end_line":400},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0","score":3.8888366,"snippet":"# ALTAROC — CATALOGUE COMPLET DES PRODUITS\n*Dernière mise à jour : Mai 2026 | Source : altaroc.pe*\n\n---\n\n## PRÉSENTATION GÉNÉRALE D'ALTAROC\n\n**Altaroc** (anciennement Amboise Partners, fondée en 1972)","reference_id":2,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":1,"end_line":52},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5","score":2.0682943,"snippet":")\n- **Format** : RAIF\n- **Ouverture à la souscription** : 5 mai 2026\n- **Statut** : Ouvert\n- Stratégie alignée sur Odyssey 2026\n\n---\n\n## GAMME 3 — DISCOVERY (Assurance Vie & PER)\n\n### Concept\nLa gamme","reference_id":3,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":278,"end_line":337},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7","score":1.6751634,"snippet":" des solutions **sur mesure** :\n- Construction de portefeuilles personnalisés\n- Accès à des gérants spécifiques non disponibles dans les millésimes standards\n- Accompagnement dédié par les équipes d'i","reference_id":4,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":400,"end_line":400},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4","score":1.3554758,"snippet":" AUM, 380+ personnes)\n  - **Hg Saturn 4** — LBO Large Cap logiciels, Europe + US (même gérant Hg)\n  - **Insight XIII + GBF XIII** — Growth Equity logiciels à forte croissance (fondé 1995, 80 Mds$ AUM,","reference_id":5,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":202,"end_line":278},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1","score":0.87912405,"snippet":" | Perpétuel | Multi-pays |\n\n---\n\n## GAMME 1 — ODYSSEY (Millésimes FPCI Français)\n\n### Concept de base\nLes millésimes Odyssey sont des **Fonds Professionnels de Capital Investissement (FPCI)** non sou","reference_id":6,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":52,"end_line":105},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3","score":0.46388084,"snippet":"xpertise logiciels\n  - **New Mountain Capital VII** — Buy-Out nord-américain (tech, santé, services)\n  - **Inflexion Partnership Capital III** — Buy-Out minoritaire, Europe du Nord\n  - **Main Capital ","reference_id":7,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":152,"end_line":202},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2","score":0.17031896,"snippet":"ers** — Growth, logiciels en forte croissance\n  - **Apax Partners** — Buy-Out global, multi-secteurs\n  - **Nordic Capital** — Buy-Out pan-européen (santé, tech)\n  - **Seven2** — Buy-Out mid-market eur","reference_id":8,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":105,"end_line":152},{"source_type":"entity","id":"ODYSSEY_2021","score":0.0,"snippet":"FPCI investment product from 2021, closed with 180M€ raised","reference_id":9,"entity_type":"PRODUCT","degree":71,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"INSIGHT_PARTNERS","score":0.0,"snippet":"Growth equity investment firm focused on software","reference_id":10,"entity_type":"ORGANIZATION","degree":38,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ALTAROC","score":0.0,"snippet":"Société française de gestion spécialisée dans le Private Equity pour investisseurs privés avertis.","reference_id":11,"entity_type":"ORGANIZATION","degree":30,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2026","score":0.0,"snippet":"6th vintage of Altaroc's Odyssey fund series","reference_id":12,"entity_type":"EVENT","degree":25,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"GENERAL_ATLANTIC","score":0.0,"snippet":"Global growth equity investment firm","reference_id":13,"entity_type":"ORGANIZATION","degree":25,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ODYSSEY","score":0.0,"snippet":"Gamme de produits d'Altaroc basée sur des millésimes FPCI annuels.","reference_id":14,"entity_type":"PRODUCT","degree":20,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"ODYSSEY_2022","score":0.0,"snippet":"FPCI investment product from 2022, closed with 465M€ raised","reference_id":15,"entity_type":"PRODUCT","degree":19,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"ALTAROC_DISCOVERY_FCPR","score":0.0,"snippet":"FCPR Evergreen ouvert en permanence accessible via plusieurs supports d'investissement","reference_id":16,"entity_type":"PRODUCT","degree":16,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"]},{"source_type":"entity","id":"ODYSSEY_2023","score":0.0,"snippet":"FPCI investment product from 2023, closed with 315M€ raised","reference_id":17,"entity_type":"PRODUCT","degree":14,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"ALTALIFE_2023","score":0.0,"snippet":"FCPR millésimé et clôturé accessible via assurance vie française et PER","reference_id":18,"entity_type":"PRODUCT","degree":13,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"]},{"source_type":"entity","id":"ALTAROC_HORIZON_2025","score":0.0,"snippet":"Closed RAIF fund targeting 50M€","reference_id":19,"entity_type":"EVENT","degree":13,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2025","score":0.0,"snippet":"5th vintage of ALTAROC ODYSSEY investment fund","reference_id":20,"entity_type":"EVENT","degree":8,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"NORDIC_CAPITAL","score":0.0,"snippet":"Pan-European buyout firm focused on healthcare and technology","reference_id":21,"entity_type":"ORGANIZATION","degree":8,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"PROGRAMME_RE-UP","score":0.0,"snippet":"Stratégie d'investissement patrimonial long terme d'Altaroc","reference_id":22,"entity_type":"CONCEPT","degree":8,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6"]},{"source_type":"entity","id":"HG","score":0.0,"snippet":"Buy-out firm focused on software in Europe","reference_id":23,"entity_type":"ORGANIZATION","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"Second vintage of Altaroc Odyssey fund with 465M€ raised","reference_id":24,"entity_type":"EVENT","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2024","score":0.0,"snippet":"4th vintage of ALTAROC ODYSSEY investment fund","reference_id":25,"entity_type":"EVENT","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"ODYSSEY_2024","score":0.0,"snippet":"FPCI investment product from 2024, closed with 355M€ raised","reference_id":26,"entity_type":"PRODUCT","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"AMF","score":0.0,"snippet":"Autorité des Marchés Financiers, régulateur français des marchés financiers.","reference_id":27,"entity_type":"ORGANIZATION","degree":6,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"APAX","score":0.0,"snippet":"Global buy-out firm","reference_id":28,"entity_type":"ORGANIZATION","degree":6,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF","score":0.0,"snippet":"Fonds secondaire evergreen multi-pays de la gamme Global Evergreen.","reference_id":29,"entity_type":"PRODUCT","degree":6,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"BRIDGEPOINT","score":0.0,"snippet":"Pan-European buy-out firm","reference_id":30,"entity_type":"ORGANIZATION","degree":5,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"AMCS","score":0.0,"snippet":"Company involved in co-investment (last signed in 2024)","reference_id":31,"entity_type":"ORGANIZATION","degree":4,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ODYSSEY_2026","score":0.0,"snippet":"Strategic event or initiative referenced in the text","reference_id":32,"entity_type":"EVENT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"]},{"source_type":"entity","id":"ALTAROC_HORIZON_2026","score":0.0,"snippet":"Open RAIF fund with subscription opening on 5 May 2026","reference_id":33,"entity_type":"EVENT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"ALTAROC_HORIZON_2024","score":0.0,"snippet":"Closed ELTIF fund with 8M€ raised","reference_id":34,"entity_type":"EVENT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"XPERTISE_LOGICIELS","score":0.0,"snippet":"Firm managing investment funds","reference_id":35,"entity_type":"ORGANIZATION","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"DISCOVERY","score":0.0,"snippet":"Gamme de produits d'Altaroc incluant des fonds FCPR et secondaires evergreen.","reference_id":36,"entity_type":"PRODUCT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"APAX_PARTNERS","score":0.0,"snippet":"Global buyout investment firm","reference_id":37,"entity_type":"ORGANIZATION","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"SEVEN2","score":0.0,"snippet":"European mid-market buyout firm","reference_id":38,"entity_type":"ORGANIZATION","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ALTAROC_DISCOVERY","score":0.0,"snippet":"Private Equity access product within French life insurance and PER contracts","reference_id":39,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"ACCEL-KKR","score":0.0,"snippet":"US-based buy-out firm focused on software growth","reference_id":40,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ONEBRIEF","score":0.0,"snippet":"Defense software company valued at $1.1B","reference_id":41,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"QLIK","score":0.0,"snippet":"Business intelligence/data analytics company","reference_id":42,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"CLEERLY","score":0.0,"snippet":"AI cardiovascular company","reference_id":43,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"AUTHORITY_BRANDS","score":0.0,"snippet":"US-based home services franchises company","reference_id":44,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"SFDR_ARTICLE_8","score":0.0,"snippet":"Sustainable Finance Disclosure Regulation classification for funds promoting environmental or social characteristics","reference_id":45,"entity_type":"LAW_NODE","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"HORIZON_2024","score":0.0,"snippet":"ELTIF investment product from 2024, closed with 8M€ raised","reference_id":46,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"MILLÉSIMES_RAIF_(HORIZON)","score":0.0,"snippet":"Fonds de fonds primaires proposés dans la gamme Odyssey pour investisseurs internationaux.","reference_id":47,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"ODYSSEY_2025","score":0.0,"snippet":"FPCI investment product from 2025, closed with 300M€ raised","reference_id":48,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"HORIZON_2025","score":0.0,"snippet":"RAIF investment product from 2025, closed with 8M€ raised","reference_id":49,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"MILLÉSIMES_FPCI_ANNUELS","score":0.0,"snippet":"Fonds de fonds primaires proposés dans la gamme Odyssey.","reference_id":50,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"HORIZON_2026","score":0.0,"snippet":"RAIF investment product from 2026, currently open","reference_id":51,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"SPECTER_OPS","score":0.0,"snippet":"Cybersecurity company","reference_id":52,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"STG_(SOFTWARE_TECHNOLOGY_GROUP)","score":0.0,"snippet":"US-based buy-out firm focused on software value","reference_id":53,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"CRESSEY_&_COMPANY","score":0.0,"snippet":"US-based buy-out firm focused on healthcare","reference_id":54,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"SCA-SICAV-RAIF","score":0.0,"snippet":"Structure juridique d'investissement (Société en Commandite par Actions, Société d'Investissement à Capital Variable, RAIF).","reference_id":55,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"INFLEXION","score":0.0,"snippet":"Private equity firm in selection universe","reference_id":56,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"FCPR","score":0.0,"snippet":"Fonds Communs de Placement à Risques, structure d'investissement en Private Equity.","reference_id":57,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"GBF_XIII","score":0.0,"snippet":"Growth Equity fund founded in 1995 with $80B AUM","reference_id":58,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"RAIF","score":0.0,"snippet":"Reserved Alternative Investment Fund, structure d'investissement luxembourgeoise.","reference_id":59,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"INSIGHT_PARTNERS_XIII","score":0.0,"snippet":"Growth Equity fund focused on high-growth software","reference_id":60,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"TROPHÉE_PATRIMONIA","score":0.0,"snippet":"Prix récompensant la meilleure relation auprès des CGP en 2024.","reference_id":61,"entity_type":"EVENT","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"IFS","score":0.0,"snippet":"European industrial ERP company","reference_id":62,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ACCESS_GROUP","score":0.0,"snippet":"UK-based mid-market software company","reference_id":63,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"FPCI","score":0.0,"snippet":"Fonds Professionnels de Capital Investissement, structure d'investissement en Private Equity.","reference_id":64,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"DISCOVERY_FCPR_(EVERGREEN)","score":0.0,"snippet":"Fonds secondaire evergreen de la gamme Discovery accessible via assurance vie/PER/PEA-PME.","reference_id":65,"entity_type":"PRODUCT","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS","score":0.0,"snippet":"Buy-out fund for large cap/upper mid-market software in North America","reference_id":66,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"ALTALIFE_2023_(FCPR)","score":0.0,"snippet":"Fonds de fonds primaire de la gamme Discovery pour assurance vie/PER en France.","reference_id":67,"entity_type":"PRODUCT","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"TA_ASSOCIATES_XV","score":0.0,"snippet":"Global growth/buy-out firm with strong software expertise","reference_id":68,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"relationship","id":"ALTAROC->ODYSSEY","score":0.0,"snippet":"ALTAROC OFFERS ODYSSEY","reference_id":69},{"source_type":"relationship","id":"ALTAROC->DISCOVERY","score":0.0,"snippet":"ALTAROC OFFERS DISCOVERY","reference_id":70},{"source_type":"relationship","id":"ODYSSEY_2022->HG","score":0.0,"snippet":"ODYSSEY_2022 INVESTS_IN HG","reference_id":71},{"source_type":"relationship","id":"ODYSSEY_2021->NORDIC_CAPITAL","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN NORDIC_CAPITAL","reference_id":72},{"source_type":"relationship","id":"ODYSSEY->MILLÉSIMES_FPCI_ANNUELS","score":0.0,"snippet":"ODYSSEY INCLUDES MILLÉSIMES_FPCI_ANNUELS","reference_id":73},{"source_type":"relationship","id":"ODYSSEY->MILLÉSIMES_RAIF_(HORIZON)","score":0.0,"snippet":"ODYSSEY INCLUDES MILLÉSIMES_RAIF_(HORIZON)","reference_id":74},{"source_type":"relationship","id":"DISCOVERY->ALTALIFE_2023_(FCPR)","score":0.0,"snippet":"DISCOVERY INCLUDES ALTALIFE_2023_(FCPR)","reference_id":75},{"source_type":"relationship","id":"DISCOVERY->DISCOVERY_FCPR_(EVERGREEN)","score":0.0,"snippet":"DISCOVERY INCLUDES DISCOVERY_FCPR_(EVERGREEN)","reference_id":76},{"source_type":"relationship","id":"MILLÉSIMES_FPCI_ANNUELS->FPCI","score":0.0,"snippet":"MILLÉSIMES_FPCI_ANNUELS IS_A FPCI","reference_id":77},{"source_type":"relationship","id":"MILLÉSIMES_RAIF_(HORIZON)->RAIF","score":0.0,"snippet":"MILLÉSIMES_RAIF_(HORIZON) IS_A RAIF","reference_id":78},{"source_type":"relationship","id":"ALTALIFE_2023_(FCPR)->FCPR","score":0.0,"snippet":"ALTALIFE_2023_(FCPR) IS_A FCPR","reference_id":79},{"source_type":"relationship","id":"DISCOVERY_FCPR_(EVERGREEN)->FCPR","score":0.0,"snippet":"DISCOVERY_FCPR_(EVERGREEN) IS_A FCPR","reference_id":80},{"source_type":"relationship","id":"ODYSSEY_2021->INSIGHT_PARTNERS","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN INSIGHT_PARTNERS","reference_id":81},{"source_type":"relationship","id":"ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF->SCA-SICAV-RAIF","score":0.0,"snippet":"ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF IS_A SCA-SICAV-RAIF","reference_id":82},{"source_type":"relationship","id":"ODYSSEY_2021->GENERAL_ATLANTIC","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN GENERAL_ATLANTIC","reference_id":83},{"source_type":"relationship","id":"GENERAL_ATLANTIC->ODYSSEY_2021","score":0.0,"snippet":"GENERAL_ATLANTIC INVESTS_IN ODYSSEY_2021","reference_id":84},{"source_type":"relationship","id":"ALTAROC->TROPHÉE_PATRIMONIA","score":0.0,"snippet":"ALTAROC RECEIVED_AWARD TROPHÉE_PATRIMONIA","reference_id":85},{"source_type":"relationship","id":"INSIGHT_PARTNERS->ODYSSEY_2021","score":0.0,"snippet":"INSIGHT_PARTNERS INVESTS_IN ODYSSEY_2021","reference_id":86},{"source_type":"relationship","id":"MILLÉSIMES_FPCI_ANNUELS->AMF","score":0.0,"snippet":"MILLÉSIMES_FPCI_ANNUELS NOT_SUBJECT_TO AMF","reference_id":87},{"source_type":"relationship","id":"ODYSSEY->FPCI","score":0.0,"snippet":"ODYSSEY IS_A FPCI","reference_id":88},{"source_type":"relationship","id":"FPCI->AMF","score":0.0,"snippet":"FPCI NOT_SUBJECT_TO AMF","reference_id":89},{"source_type":"relationship","id":"ODYSSEY->GENERAL_ATLANTIC","score":0.0,"snippet":"ODYSSEY INVESTS_WITH GENERAL_ATLANTIC","reference_id":90},{"source_type":"relationship","id":"ONEBRIEF->ODYSSEY_2025","score":0.0,"snippet":"ONEBRIEF INVESTMENT ODYSSEY_2025","reference_id":91},{"source_type":"relationship","id":"ODYSSEY->INSIGHT_PARTNERS","score":0.0,"snippet":"ODYSSEY INVESTS_WITH INSIGHT_PARTNERS","reference_id":92},{"source_type":"relationship","id":"ODYSSEY->APAX_PARTNERS","score":0.0,"snippet":"ODYSSEY INVESTS_WITH APAX_PARTNERS","reference_id":93},{"source_type":"relationship","id":"ONEBRIEF->INSIGHT_PARTNERS","score":0.0,"snippet":"ONEBRIEF INVESTOR INSIGHT_PARTNERS","reference_id":94},{"source_type":"relationship","id":"ODYSSEY->NORDIC_CAPITAL","score":0.0,"snippet":"ODYSSEY INVESTS_WITH NORDIC_CAPITAL","reference_id":95},{"source_type":"relationship","id":"ODYSSEY->SEVEN2","score":0.0,"snippet":"ODYSSEY INVESTS_WITH SEVEN2","reference_id":96},{"source_type":"relationship","id":"AMCS->ODYSSEY_2021","score":0.0,"snippet":"AMCS INVESTS_IN ODYSSEY_2021","reference_id":97},{"source_type":"relationship","id":"QLIK->ODYSSEY_2024","score":0.0,"snippet":"QLIK INVESTS_IN ODYSSEY_2024","reference_id":98},{"source_type":"relationship","id":"ODYSSEY_2022->APAX","score":0.0,"snippet":"ODYSSEY_2022 INVESTS_IN APAX","reference_id":99},{"source_type":"relationship","id":"ODYSSEY_2022->BRIDGEPOINT","score":0.0,"snippet":"ODYSSEY_2022 INVESTS_IN BRIDGEPOINT","reference_id":100},{"source_type":"relationship","id":"APAX_PARTNERS->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"APAX_PARTNERS INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":101},{"source_type":"relationship","id":"HG->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"HG INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":102},{"source_type":"relationship","id":"BRIDGEPOINT->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"BRIDGEPOINT INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":103},{"source_type":"relationship","id":"CRESSEY_&_COMPANY->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"CRESSEY_&_COMPANY INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":104},{"source_type":"relationship","id":"STG_(SOFTWARE_TECHNOLOGY_GROUP)->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"STG_(SOFTWARE_TECHNOLOGY_GROUP) INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":105},{"source_type":"relationship","id":"ACCEL-KKR->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"ACCEL-KKR INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":106},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2022->ACCESS_GROUP","score":0.0,"snippet":"ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH ACCESS_GROUP","reference_id":107},{"source_type":"relationship","id":"ODYSSEY_2021->HG","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN HG","reference_id":108},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2022->AUTHORITY_BRANDS","score":0.0,"snippet":"ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH AUTHORITY_BRANDS","reference_id":109},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2022->IFS","score":0.0,"snippet":"ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH IFS","reference_id":110},{"source_type":"relationship","id":"APAX_PARTNERS->AUTHORITY_BRANDS","score":0.0,"snippet":"APAX_PARTNERS INVESTED_IN AUTHORITY_BRANDS","reference_id":111},{"source_type":"relationship","id":"HG->ACCESS_GROUP","score":0.0,"snippet":"HG INVESTED_IN ACCESS_GROUP","reference_id":112},{"source_type":"relationship","id":"HG->IFS","score":0.0,"snippet":"HG INVESTED_IN IFS","reference_id":113},{"source_type":"relationship","id":"ODYSSEY_2021->APAX","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN APAX","reference_id":114},{"source_type":"relationship","id":"SFDR_ARTICLE_8->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"SFDR_ARTICLE_8 CLASSIFIED_AS ALTAROC_ODYSSEY_2022","reference_id":115},{"source_type":"relationship","id":"AMCS->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"AMCS CO_INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":116},{"source_type":"relationship","id":"NORDIC_CAPITAL->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"NORDIC_CAPITAL CO_INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":117},{"source_type":"relationship","id":"SEVEN2->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"SEVEN2 CO_INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":118},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2024->FPCI","score":0.0,"snippet":"ALTAROC_ODYSSEY_2024 USES FPCI","reference_id":119},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2024->THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS","score":0.0,"snippet":"ALTAROC_ODYSSEY_2024 INVESTS_IN THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS","reference_id":120},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2024->QLIK","score":0.0,"snippet":"ALTAROC_ODYSSEY_2024 CO_INVESTS_WITH QLIK","reference_id":121},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2025->FPCI","score":0.0,"snippet":"ALTAROC_ODYSSEY_2025 USES FPCI","reference_id":122},{"source_type":"relationship","id":"ALTAROC->ODYSSEY_2023","score":0.0,"snippet":"ALTAROC OFFERS ODYSSEY_2023","reference_id":123},{"source_type":"relationship","id":"INSIGHT_PARTNERS_XIII->ALTAROC_ODYSSEY_2026","score":0.0,"snippet":"INSIGHT_PARTNERS_XIII MANAGES ALTAROC_ODYSSEY_2026","reference_id":124},{"source_type":"relationship","id":"GBF_XIII->ALTAROC_ODYSSEY_2026","score":0.0,"snippet":"GBF_XIII MANAGES ALTAROC_ODYSSEY_2026","reference_id":125},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2026->INSIGHT_PARTNERS_XIII","score":0.0,"snippet":"ALTAROC_ODYSSEY_2026 INCLUDES INSIGHT_PARTNERS_XIII","reference_id":126},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2026->INFLEXION","score":0.0,"snippet":"ALTAROC_ODYSSEY_2026 INCLUDES INFLEXION","reference_id":127},{"source_type":"relationship","id":"ODYSSEY_2021->SEVEN2","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN SEVEN2","reference_id":128}],"query_mode":"global","retrieval_time_ms":632}

data: {"type":"token","content":""}

data: {"type":"token","content":""}

data: {"type":"token","content":"Vo"}

data: {"type":"token","content":"ici les"}

data: {"type":"token","content":" informations disponibles"}

data: {"type":"token","content":" concernant"}

data: {"type":"token","content":" le **"}

data: {"type":"token","content":"produit Odyssey "}

data: {"type":"token","content":"2026"}

data: {"type":"token","content":" d"}

data: {"type":"token","content":"'Altaroc"}

data: {"type":"token","content":"** :\n\n---\n\n###"}

data: {"type":"token","content":" **Od"}

data: {"type":"token","content":"yssey"}

data: {"type":"token","content":" 202"}

data: {"type":"token","content":"6 (Alt"}

data: {"type":"token","content":"aroc)**\n"}

data: {"type":"token","content":"- **Type**"}

data: {"type":"token","content":" : **6"}

data: {"type":"token","content":"ᵉ millés"}

data: {"type":"token","content":"ime** de la"}

data: {"type":"token","content":" gamme **Odys"}

data: {"type":"token","content":"sey**,"}

data: {"type":"token","content":" une série"}

data: {"type":"token","content":" de **"}

data: {"type":"token","content":"Fonds Professionnels"}

data: {"type":"token","content":" de Capital Investissement"}

data: {"type":"token","content":" (FPCI)**"}

data: {"type":"token","content":".\n"}

data: {"type":"token","content":"- **Format**"}

data: {"type":"token","content":" : **Soci"}

data: {"type":"token","content":"été de Libre Part"}

data: {"type":"token","content":"enariat (SL"}

data: {"type":"token","content":"P)** (structure"}

data: {"type":"token","content":" juridique flexible"}

data: {"type":"token","content":" pour"}

data: {"type":"token","content":" invest"}

data: {"type":"token","content":"isseurs professionnels"}

data: {"type":"token","content":").\n- **Stat"}

data: {"type":"token","content":"ut** : **"}

data: {"type":"token","content":"Ouvert à"}

data: {"type":"token","content":" la souscription depuis"}

data: {"type":"token","content":" le 5 mai"}

data: {"type":"token","content":" 202"}

data: {"type":"token","content":"6**.\n- **"}

data: {"type":"token","content":"Minimum de souscription"}

data: {"type":"token","content":"** : **1"}

data: {"type":"token","content":"00 0"}

data: {"type":"token","content":"00 €**,"}

data: {"type":"token","content":" avec des"}

data: {"type":"token","content":" appels de"}

data: {"type":"token","content":" fonds éche"}

data: {"type":"token","content":"lonnés à **"}

data: {"type":"token","content":"10% par"}

data: {"type":"token","content":" semestre pendant"}

data: {"type":"token","content":" 5 ans**.\n"}

data: {"type":"token","content":"- **Dur"}

data: {"type":"token","content":"ée** : **"}

data: {"type":"token","content":"10 ans**"}

data: {"type":"token","content":" (eng"}

data: {"type":"token","content":"agement long"}

data: {"type":"token","content":" terme).\n- **"}

data: {"type":"token","content":"ES"}

data: {"type":"token","content":"G** : Class"}

data: {"type":"token","content":"é"}

data: {"type":"token","content":" **Article 8"}

data: {"type":"token","content":"**"}

data: {"type":"token","content":" selon"}

data: {"type":"token","content":" le"}

data: {"type":"token","content":" **SFDR**"}

data: {"type":"token","content":" (f"}

data: {"type":"token","content":"onds promou"}

data: {"type":"token","content":"vant des caractéristiques"}

data: {"type":"token","content":" environnementales ou"}

data: {"type":"token","content":" sociales).\n- **"}

data: {"type":"token","content":"Stratégie"}

data: {"type":"token","content":"** :\n"}

data: {"type":"token","content":"  - **8"}

data: {"type":"token","content":"0% invest"}

data: {"type":"token","content":"i"}

data: {"type":"token","content":" dans "}

data: {"type":"token","content":"5 à 7"}

data: {"type":"token","content":" fonds primaires**"}

data: {"type":"token","content":" de Private Equity internationaux"}

data: {"type":"token","content":" (G"}

data: {"type":"token","content":"rowth Equity et"}

data: {"type":"token","content":" Buy-Out).\n"}

data: {"type":"token","content":"  - **2"}

data: {"type":"token","content":"0% en"}

data: {"type":"token","content":" co-investissements"}

data: {"type":"token","content":"** aux"}

data: {"type":"token","content":" côtés des mêmes"}

data: {"type":"token","content":" gérants ("}

data: {"type":"token","content":"sans frais supplémentaires"}

data: {"type":"token","content":").\n"}

data: {"type":"token","content":"  - **S"}

data: {"type":"token","content":"ecteurs cibles"}

data: {"type":"token","content":"** : **"}

data: {"type":"token","content":"50-6"}

data: {"type":"token","content":"0% logiciels"}

data: {"type":"token","content":"/tech**, "}

data: {"type":"token","content":"10-2"}

data: {"type":"token","content":"0% santé,"}

data: {"type":"token","content":" ~20%"}

data: {"type":"token","content":" services, ~"}

data: {"type":"token","content":"10% consommation"}

data: {"type":"token","content":".\n "}

data: {"type":"token","content":" - **Gé"}

data: {"type":"token","content":"ographies** : Europe"}

data: {"type":"token","content":" (~45%),"}

data: {"type":"token","content":" Amérique du"}

data: {"type":"token","content":" Nord (~45"}

data: {"type":"token","content":"%), Reste du monde"}

data: {"type":"token","content":" (~10%"}

data: {"type":"token","content":").\n- **G"}

data: {"type":"token","content":"érants déjà"}

data: {"type":"token","content":" sélectionnés** :\n"}

data: {"type":"token","content":"  - **Ins"}

data: {"type":"token","content":"ight Partners XIII**"}

data: {"type":"token","content":" (Growth Equity"}

data: {"type":"token","content":", logiciels en"}

data: {"type":"token","content":" forte croissance).\n "}

data: {"type":"token","content":" - **Hg Saturn"}

data: {"type":"token","content":" 4** ("}

data: {"type":"token","content":"LBO Large Cap"}

data: {"type":"token","content":" logic"}

data: {"type":"token","content":"iels, Europe +"}

data: {"type":"token","content":" US"}

data: {"type":"token","content":").\n- **G"}

data: {"type":"token","content":"érants en analyse"}

data: {"type":"token","content":" avanc"}

data: {"type":"token","content":"ée** ("}

data: {"type":"token","content":"5"}

data: {"type":"token","content":" autres) :\n "}

data: {"type":"token","content":" - Inf"}

data: {"type":"token","content":"lexion, Great"}

data: {"type":"token","content":" Hill Partners, Nordic"}

data: {"type":"token","content":" Capital XII, New"}

data: {"type":"token","content":" Mountain Strategic Equity Fund"}

data: {"type":"token","content":" II, Hg Mercury"}

data: {"type":"token","content":" 5, K"}

data: {"type":"token","content":"1 Investment Management ("}

data: {"type":"token","content":"K6).\n-"}

data: {"type":"token","content":" **Objectifs de"}

data: {"type":"token","content":" performance** ("}

data: {"type":"token","content":"non garantis)"}

data: {"type":"token","content":" :\n  - **"}

data: {"type":"token","content":"TRI cible** :"}

data: {"type":"token","content":" **13%"}

data: {"type":"token","content":" par an**"}

data: {"type":"token","content":" avant fiscalité.\n "}

data: {"type":"token","content":" - **Multiple cible"}

data: {"type":"token","content":"** : **1"}

data: {"type":"token","content":",7x**"}

data: {"type":"token","content":" l'engagement avant"}

data: {"type":"token","content":" fiscalité.\n- **"}

data: {"type":"token","content":"Frais** :\n"}

data: {"type":"token","content":"  - **F"}

data: {"type":"token","content":"rais de gestion ann"}

data: {"type":"token","content":"uels** : Dé"}

data: {"type":"token","content":"gressifs selon"}

data: {"type":"token","content":" le montant sous"}

data: {"type":"token","content":"crit (ex."}

data: {"type":"token","content":" : 2,"}

data: {"type":"token","content":"5% pour la"}

data: {"type":"token","content":" tranche standard,"}

data: {"type":"token","content":" jusqu"}

data: {"type":"token","content":"'à 1,"}

data: {"type":"token","content":"65% pour"}

data: {"type":"token","content":" les tran"}

data: {"type":"token","content":"ches élevées).\n"}

data: {"type":"token","content":"  - **Com"}

data: {"type":"token","content":"mission de performance**"}

data: {"type":"token","content":" : **20"}

data: {"type":"token","content":"%**"}

data: {"type":"token","content":" exclusivement"}

data: {"type":"token","content":" sur les co-in"}

data: {"type":"token","content":"vestissements (pas"}

data: {"type":"token","content":" sur la part fonds"}

data: {"type":"token","content":").\n\n---\n"}

data: {"type":"token","content":"### **Diff"}

data: {"type":"token","content":"érences"}

data: {"type":"token","content":" clés"}

data: {"type":"token","content":" avec les"}

data: {"type":"token","content":" mill"}

data: {"type":"token","content":"ésimes"}

data: {"type":"token","content":" précédents**\n- **"}

data: {"type":"token","content":"Nou"}

data: {"type":"token","content":"velle"}

data: {"type":"token","content":" structure juridique"}

data: {"type":"token","content":"** : Passage"}

data: {"type":"token","content":" du"}

data: {"type":"token","content":" **FPCI**"}

data: {"type":"token","content":" ("}

data: {"type":"token","content":"non"}

data: {"type":"token","content":" soumis à l'A"}

data: {"type":"token","content":"MF"}

data: {"type":"token","content":") à"}

data: {"type":"token","content":" la **SLP"}

data: {"type":"token","content":"** (plus"}

data: {"type":"token","content":" flexible pour les invest"}

data: {"type":"token","content":"isseurs internationaux).\n-"}

data: {"type":"token","content":" **Univers"}

data: {"type":"token","content":" de"}

data: {"type":"token","content":" sélection élargi"}

data: {"type":"token","content":"** : "}

data: {"type":"token","content":"7"}

data: {"type":"token","content":" fonds"}

data: {"type":"token","content":" cibles (contre"}

data: {"type":"token","content":" 5"}

data: {"type":"token","content":"-"}

data: {"type":"token","content":"6 pour les mill"}

data: {"type":"token","content":"ésimes"}

data: {"type":"token","content":" précédents).\n- **"}

data: {"type":"token","content":"Ouverture internationale"}

data: {"type":"token","content":"** : Bien"}

data: {"type":"token","content":" que la gamme"}

data: {"type":"token","content":" Odyssey"}

data: {"type":"token","content":" reste"}

data: {"type":"token","content":" principalement"}

data: {"type":"token","content":" française"}

data: {"type":"token","content":", la"}

data: {"type":"token","content":" S"}

data: {"type":"token","content":"LP permet une"}

data: {"type":"token","content":" plus"}

data: {"type":"token","content":" grande"}

data: {"type":"token","content":" flexibilité pour les"}

data: {"type":"token","content":" investisseurs étrangers.\n\n"}

data: {"type":"token","content":"---\n### **Pour"}

data: {"type":"token","content":" qui"}

data: {"type":"token","content":" ?**\n- **"}

data: {"type":"token","content":"Investisseurs privés avert"}

data: {"type":"token","content":"is** (minimum"}

data: {"type":"token","content":" "}

data: {"type":"token","content":"100 "}

data: {"type":"token","content":"000 €"}

data: {"type":"token","content":").\n- **Hor"}

data: {"type":"token","content":"izon d"}

data: {"type":"token","content":"'investissement long"}

data: {"type":"token","content":" terme** (1"}

data: {"type":"token","content":"0 ans).\n-"}

data: {"type":"token","content":" **Acc"}

data: {"type":"token","content":"ès à une"}

data: {"type":"token","content":" stratégie divers"}

data: {"type":"token","content":"ifiée** avec"}

data: {"type":"token","content":" des g"}

data: {"type":"token","content":"érants de premier"}

data: {"type":"token","content":" plan (ex"}

data: {"type":"token","content":". : Hg,"}

data: {"type":"token","content":" Ins"}

data: {"type":"token","content":"ight Partners).\n\n---\n"}

data: {"type":"token","content":"### **O"}

data: {"type":"token","content":"ù sous"}

data: {"type":"token","content":"crire ?**\nVia"}

data: {"type":"token","content":" les"}

data: {"type":"token","content":" **can"}

data: {"type":"token","content":"aux de distribution d"}

data: {"type":"token","content":"'Altaroc"}

data: {"type":"token","content":"** :\n- Conse"}

data: {"type":"token","content":"illers en G"}

data: {"type":"token","content":"estion de Patrimoine"}

data: {"type":"token","content":" (C"}

data: {"type":"token","content":"GP).\n- Ban"}

data: {"type":"token","content":"ques privées.\n"}

data: {"type":"token","content":"- Family offices.\n"}

data: {"type":"token","content":"- T"}

data: {"type":"token","content":"iers-gérants"}

data: {"type":"token","content":".\n\n---\n*"}

data: {"type":"token","content":"Source : Con"}

data: {"type":"token","content":"texte Altar"}

data: {"type":"token","content":"oc ("}

data: {"type":"token","content":"2026"}

data: {"type":"token","content":"),"}

data: {"type":"token","content":" Document"}

data: {"type":"token","content":" Chunks ["}

data: {"type":"token","content":"5"}

data: {"type":"token","content":"] et [6"}

data: {"type":"token","content":"].*\n\n"}

data: {"type":"token","content":"---\n**Note**"}

data: {"type":"token","content":" : Les performances pass"}

data: {"type":"token","content":"ées ne garant"}

data: {"type":"token","content":"issent pas les résultats"}

data: {"type":"token","content":" futurs. L"}

data: {"type":"token","content":"'investissement en Private"}

data: {"type":"token","content":" Equity comporte"}

data: {"type":"token","content":" des risques de perte"}

data: {"type":"token","content":" en capital et d"}

data: {"type":"token","content":"'illiquidité."}

data: {"type":"token","content":""}

data: {"type":"done","assistant_message_id":"6c480290-3baf-449b-921b-3ead4dd70ce5","tokens_used":780,"duration_ms":6384,"llm_provider":"mistral","llm_model":"mistral-small-latest"}

data: {"type":"title_update","conversation_id":"88d2fc80-5d22-49ee-a557-f3662419dadf","title":"Odyssey 2026 Altaroc Product"}





---------

{
    "conversation": {
        "id": "88d2fc80-5d22-49ee-a557-f3662419dadf",
        "tenant_id": "b9d60c6e-c568-487a-a3b9-27318da1549d",
        "workspace_id": "4382e7ab-6fb6-48b6-b49c-c3d1841dcc69",
        "title": "Odyssey 2026 Altaroc Product",
        "mode": "global",
        "is_pinned": false,
        "is_archived": false,
        "folder_id": null,
        "share_id": null,
        "message_count": null,
        "last_message_preview": null,
        "created_at": "2026-06-29T13:47:30.357878+00:00",
        "updated_at": "2026-06-29T13:47:37.274637+00:00"
    },
    "messages": [
        {
            "id": "52284ae9-d1ae-4897-9840-1d14d0935ff2",
            "conversation_id": "88d2fc80-5d22-49ee-a557-f3662419dadf",
            "parent_id": null,
            "role": "user",
            "content": "c'est quoi le produit odyssey 2026 altaroc ?",
            "mode": null,
            "tokens_used": null,
            "duration_ms": null,
            "thinking_time_ms": null,
            "context": null,
            "is_error": false,
            "created_at": "2026-06-29T13:47:30.360692+00:00",
            "updated_at": "2026-06-29T13:47:30.360692+00:00"
        },
        {
            "id": "6c480290-3baf-449b-921b-3ead4dd70ce5",
            "conversation_id": "88d2fc80-5d22-49ee-a557-f3662419dadf",
            "parent_id": "52284ae9-d1ae-4897-9840-1d14d0935ff2",
            "role": "assistant",
            "content": "Voici les informations disponibles concernant le **produit Odyssey 2026 d'Altaroc** :\n\n---\n\n### **Odyssey 2026 (Altaroc)**\n- **Type** : **6ᵉ millésime** de la gamme **Odyssey**, une série de **Fonds Professionnels de Capital Investissement (FPCI)**.\n- **Format** : **Société de Libre Partenariat (SLP)** (structure juridique flexible pour investisseurs professionnels).\n- **Statut** : **Ouvert à la souscription depuis le 5 mai 2026**.\n- **Minimum de souscription** : **100 000 €**, avec des appels de fonds échelonnés à **10% par semestre pendant 5 ans**.\n- **Durée** : **10 ans** (engagement long terme).\n- **ESG** : Classé **Article 8** selon le **SFDR** (fonds promouvant des caractéristiques environnementales ou sociales).\n- **Stratégie** :\n  - **80% investi dans 5 à 7 fonds primaires** de Private Equity internationaux (Growth Equity et Buy-Out).\n  - **20% en co-investissements** aux côtés des mêmes gérants (sans frais supplémentaires).\n  - **Secteurs cibles** : **50-60% logiciels/tech**, 10-20% santé, ~20% services, ~10% consommation.\n  - **Géographies** : Europe (~45%), Amérique du Nord (~45%), Reste du monde (~10%).\n- **Gérants déjà sélectionnés** :\n  - **Insight Partners XIII** (Growth Equity, logiciels en forte croissance).\n  - **Hg Saturn 4** (LBO Large Cap logiciels, Europe + US).\n- **Gérants en analyse avancée** (5 autres) :\n  - Inflexion, Great Hill Partners, Nordic Capital XII, New Mountain Strategic Equity Fund II, Hg Mercury 5, K1 Investment Management (K6).\n- **Objectifs de performance** (non garantis) :\n  - **TRI cible** : **13% par an** avant fiscalité.\n  - **Multiple cible** : **1,7x** l'engagement avant fiscalité.\n- **Frais** :\n  - **Frais de gestion annuels** : Dégressifs selon le montant souscrit (ex. : 2,5% pour la tranche standard, jusqu'à 1,65% pour les tranches élevées).\n  - **Commission de performance** : **20%** exclusivement sur les co-investissements (pas sur la part fonds).\n\n---\n### **Différences clés avec les millésimes précédents**\n- **Nouvelle structure juridique** : Passage du **FPCI** (non soumis à l'AMF) à la **SLP** (plus flexible pour les investisseurs internationaux).\n- **Univers de sélection élargi** : 7 fonds cibles (contre 5-6 pour les millésimes précédents).\n- **Ouverture internationale** : Bien que la gamme Odyssey reste principalement française, la SLP permet une plus grande flexibilité pour les investisseurs étrangers.\n\n---\n### **Pour qui ?**\n- **Investisseurs privés avertis** (minimum 100 000 €).\n- **Horizon d'investissement long terme** (10 ans).\n- **Accès à une stratégie diversifiée** avec des gérants de premier plan (ex. : Hg, Insight Partners).\n\n---\n### **Où souscrire ?**\nVia les **canaux de distribution d'Altaroc** :\n- Conseillers en Gestion de Patrimoine (CGP).\n- Banques privées.\n- Family offices.\n- Tiers-gérants.\n\n---\n*Source : Contexte Altaroc (2026), Document Chunks [5] et [6].*\n\n---\n**Note** : Les performances passées ne garantissent pas les résultats futurs. L'investissement en Private Equity comporte des risques de perte en capital et d'illiquidité.",
            "mode": null,
            "tokens_used": 780,
            "duration_ms": 6384,
            "thinking_time_ms": null,
            "context": {
                "sources": [
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6",
                        "title": "altaroc_produits.md",
                        "content": "res** :\n  - Société d'investissement existant depuis plus de 15 ans\n  - Historique de performance 15 ans dans le 1er quartile\n  - Taux de perte historique < 2%\n  - Taille du fonds en levée > 1 Md€ et ",
                        "score": 4.005088806152344,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0",
                        "title": "altaroc_produits.md",
                        "content": "# ALTAROC — CATALOGUE COMPLET DES PRODUITS\n*Dernière mise à jour : Mai 2026 | Source : altaroc.pe*\n\n---\n\n## PRÉSENTATION GÉNÉRALE D'ALTAROC\n\n**Altaroc** (anciennement Amboise Partners, fondée en 1972)",
                        "score": 3.888836622238159,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5",
                        "title": "altaroc_produits.md",
                        "content": ")\n- **Format** : RAIF\n- **Ouverture à la souscription** : 5 mai 2026\n- **Statut** : Ouvert\n- Stratégie alignée sur Odyssey 2026\n\n---\n\n## GAMME 3 — DISCOVERY (Assurance Vie & PER)\n\n### Concept\nLa gamme",
                        "score": 2.0682942867279053,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7",
                        "title": "altaroc_produits.md",
                        "content": " des solutions **sur mesure** :\n- Construction de portefeuilles personnalisés\n- Accès à des gérants spécifiques non disponibles dans les millésimes standards\n- Accompagnement dédié par les équipes d'i",
                        "score": 1.6751633882522583,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4",
                        "title": "altaroc_produits.md",
                        "content": " AUM, 380+ personnes)\n  - **Hg Saturn 4** — LBO Large Cap logiciels, Europe + US (même gérant Hg)\n  - **Insight XIII + GBF XIII** — Growth Equity logiciels à forte croissance (fondé 1995, 80 Mds$ AUM,",
                        "score": 1.3554757833480835,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1",
                        "title": "altaroc_produits.md",
                        "content": " | Perpétuel | Multi-pays |\n\n---\n\n## GAMME 1 — ODYSSEY (Millésimes FPCI Français)\n\n### Concept de base\nLes millésimes Odyssey sont des **Fonds Professionnels de Capital Investissement (FPCI)** non sou",
                        "score": 0.8791240453720093,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3",
                        "title": "altaroc_produits.md",
                        "content": "xpertise logiciels\n  - **New Mountain Capital VII** — Buy-Out nord-américain (tech, santé, services)\n  - **Inflexion Partnership Capital III** — Buy-Out minoritaire, Europe du Nord\n  - **Main Capital ",
                        "score": 0.46388083696365356,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2",
                        "title": "altaroc_produits.md",
                        "content": "ers** — Growth, logiciels en forte croissance\n  - **Apax Partners** — Buy-Out global, multi-secteurs\n  - **Nordic Capital** — Buy-Out pan-européen (santé, tech)\n  - **Seven2** — Buy-Out mid-market eur",
                        "score": 0.17031896114349365,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    }
                ],
                "entities": [
                    {
                        "name": "ODYSSEY_2021",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2021, closed with 180M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "INSIGHT_PARTNERS",
                        "entity_type": "ORGANIZATION",
                        "description": "Growth equity investment firm focused on software",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ALTAROC",
                        "entity_type": "ORGANIZATION",
                        "description": "Société française de gestion spécialisée dans le Private Equity pour investisseurs privés avertis.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2026",
                        "entity_type": "EVENT",
                        "description": "6th vintage of Altaroc's Odyssey fund series",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "GENERAL_ATLANTIC",
                        "entity_type": "ORGANIZATION",
                        "description": "Global growth equity investment firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ODYSSEY",
                        "entity_type": "PRODUCT",
                        "description": "Gamme de produits d'Altaroc basée sur des millésimes FPCI annuels.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2022",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2022, closed with 465M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "ALTAROC_DISCOVERY_FCPR",
                        "entity_type": "PRODUCT",
                        "description": "FCPR Evergreen ouvert en permanence accessible via plusieurs supports d'investissement",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2023",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2023, closed with 315M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "ALTALIFE_2023",
                        "entity_type": "PRODUCT",
                        "description": "FCPR millésimé et clôturé accessible via assurance vie française et PER",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"
                        ]
                    },
                    {
                        "name": "ALTAROC_HORIZON_2025",
                        "entity_type": "EVENT",
                        "description": "Closed RAIF fund targeting 50M€",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2025",
                        "entity_type": "EVENT",
                        "description": "5th vintage of ALTAROC ODYSSEY investment fund",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "NORDIC_CAPITAL",
                        "entity_type": "ORGANIZATION",
                        "description": "Pan-European buyout firm focused on healthcare and technology",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "PROGRAMME_RE-UP",
                        "entity_type": "CONCEPT",
                        "description": "Stratégie d'investissement patrimonial long terme d'Altaroc",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6"
                        ]
                    },
                    {
                        "name": "HG",
                        "entity_type": "ORGANIZATION",
                        "description": "Buy-out firm focused on software in Europe",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2022",
                        "entity_type": "EVENT",
                        "description": "Second vintage of Altaroc Odyssey fund with 465M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2024",
                        "entity_type": "EVENT",
                        "description": "4th vintage of ALTAROC ODYSSEY investment fund",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2024",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2024, closed with 355M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "AMF",
                        "entity_type": "ORGANIZATION",
                        "description": "Autorité des Marchés Financiers, régulateur français des marchés financiers.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "APAX",
                        "entity_type": "ORGANIZATION",
                        "description": "Global buy-out firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF",
                        "entity_type": "PRODUCT",
                        "description": "Fonds secondaire evergreen multi-pays de la gamme Global Evergreen.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "BRIDGEPOINT",
                        "entity_type": "ORGANIZATION",
                        "description": "Pan-European buy-out firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "AMCS",
                        "entity_type": "ORGANIZATION",
                        "description": "Company involved in co-investment (last signed in 2024)",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2026",
                        "entity_type": "EVENT",
                        "description": "Strategic event or initiative referenced in the text",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"
                        ]
                    },
                    {
                        "name": "ALTAROC_HORIZON_2026",
                        "entity_type": "EVENT",
                        "description": "Open RAIF fund with subscription opening on 5 May 2026",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "ALTAROC_HORIZON_2024",
                        "entity_type": "EVENT",
                        "description": "Closed ELTIF fund with 8M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "XPERTISE_LOGICIELS",
                        "entity_type": "ORGANIZATION",
                        "description": "Firm managing investment funds",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "DISCOVERY",
                        "entity_type": "PRODUCT",
                        "description": "Gamme de produits d'Altaroc incluant des fonds FCPR et secondaires evergreen.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "APAX_PARTNERS",
                        "entity_type": "ORGANIZATION",
                        "description": "Global buyout investment firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "SEVEN2",
                        "entity_type": "ORGANIZATION",
                        "description": "European mid-market buyout firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ALTAROC_DISCOVERY",
                        "entity_type": "PRODUCT",
                        "description": "Private Equity access product within French life insurance and PER contracts",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "ACCEL-KKR",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based buy-out firm focused on software growth",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ONEBRIEF",
                        "entity_type": "ORGANIZATION",
                        "description": "Defense software company valued at $1.1B",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "QLIK",
                        "entity_type": "ORGANIZATION",
                        "description": "Business intelligence/data analytics company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "CLEERLY",
                        "entity_type": "ORGANIZATION",
                        "description": "AI cardiovascular company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "AUTHORITY_BRANDS",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based home services franchises company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "SFDR_ARTICLE_8",
                        "entity_type": "LAW_NODE",
                        "description": "Sustainable Finance Disclosure Regulation classification for funds promoting environmental or social characteristics",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "HORIZON_2024",
                        "entity_type": "PRODUCT",
                        "description": "ELTIF investment product from 2024, closed with 8M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "MILLÉSIMES_RAIF_(HORIZON)",
                        "entity_type": "PRODUCT",
                        "description": "Fonds de fonds primaires proposés dans la gamme Odyssey pour investisseurs internationaux.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2025",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2025, closed with 300M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "HORIZON_2025",
                        "entity_type": "PRODUCT",
                        "description": "RAIF investment product from 2025, closed with 8M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "MILLÉSIMES_FPCI_ANNUELS",
                        "entity_type": "PRODUCT",
                        "description": "Fonds de fonds primaires proposés dans la gamme Odyssey.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "HORIZON_2026",
                        "entity_type": "PRODUCT",
                        "description": "RAIF investment product from 2026, currently open",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "SPECTER_OPS",
                        "entity_type": "ORGANIZATION",
                        "description": "Cybersecurity company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "STG_(SOFTWARE_TECHNOLOGY_GROUP)",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based buy-out firm focused on software value",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "CRESSEY_&_COMPANY",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based buy-out firm focused on healthcare",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "SCA-SICAV-RAIF",
                        "entity_type": "TECHNOLOGY",
                        "description": "Structure juridique d'investissement (Société en Commandite par Actions, Société d'Investissement à Capital Variable, RAIF).",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "INFLEXION",
                        "entity_type": "ORGANIZATION",
                        "description": "Private equity firm in selection universe",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "FCPR",
                        "entity_type": "TECHNOLOGY",
                        "description": "Fonds Communs de Placement à Risques, structure d'investissement en Private Equity.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "GBF_XIII",
                        "entity_type": "ORGANIZATION",
                        "description": "Growth Equity fund founded in 1995 with $80B AUM",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "RAIF",
                        "entity_type": "TECHNOLOGY",
                        "description": "Reserved Alternative Investment Fund, structure d'investissement luxembourgeoise.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "INSIGHT_PARTNERS_XIII",
                        "entity_type": "ORGANIZATION",
                        "description": "Growth Equity fund focused on high-growth software",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "TROPHÉE_PATRIMONIA",
                        "entity_type": "EVENT",
                        "description": "Prix récompensant la meilleure relation auprès des CGP en 2024.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "IFS",
                        "entity_type": "ORGANIZATION",
                        "description": "European industrial ERP company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ACCESS_GROUP",
                        "entity_type": "ORGANIZATION",
                        "description": "UK-based mid-market software company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "FPCI",
                        "entity_type": "TECHNOLOGY",
                        "description": "Fonds Professionnels de Capital Investissement, structure d'investissement en Private Equity.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "DISCOVERY_FCPR_(EVERGREEN)",
                        "entity_type": "PRODUCT",
                        "description": "Fonds secondaire evergreen de la gamme Discovery accessible via assurance vie/PER/PEA-PME.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS",
                        "entity_type": "ORGANIZATION",
                        "description": "Buy-out fund for large cap/upper mid-market software in North America",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "ALTALIFE_2023_(FCPR)",
                        "entity_type": "PRODUCT",
                        "description": "Fonds de fonds primaire de la gamme Discovery pour assurance vie/PER en France.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "TA_ASSOCIATES_XV",
                        "entity_type": "ORGANIZATION",
                        "description": "Global growth/buy-out firm with strong software expertise",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    }
                ],
                "relationships": [
                    {
                        "source": "ALTAROC",
                        "target": "ODYSSEY",
                        "relation_type": "OFFERS",
                        "description": "ALTAROC OFFERS ODYSSEY",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC",
                        "target": "DISCOVERY",
                        "relation_type": "OFFERS",
                        "description": "ALTAROC OFFERS DISCOVERY",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2022",
                        "target": "HG",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2022 INVESTS_IN HG",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "NORDIC_CAPITAL",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN NORDIC_CAPITAL",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "MILLÉSIMES_FPCI_ANNUELS",
                        "relation_type": "INCLUDES",
                        "description": "ODYSSEY INCLUDES MILLÉSIMES_FPCI_ANNUELS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "MILLÉSIMES_RAIF_(HORIZON)",
                        "relation_type": "INCLUDES",
                        "description": "ODYSSEY INCLUDES MILLÉSIMES_RAIF_(HORIZON)",
                        "score": 0.0
                    },
                    {
                        "source": "DISCOVERY",
                        "target": "ALTALIFE_2023_(FCPR)",
                        "relation_type": "INCLUDES",
                        "description": "DISCOVERY INCLUDES ALTALIFE_2023_(FCPR)",
                        "score": 0.0
                    },
                    {
                        "source": "DISCOVERY",
                        "target": "DISCOVERY_FCPR_(EVERGREEN)",
                        "relation_type": "INCLUDES",
                        "description": "DISCOVERY INCLUDES DISCOVERY_FCPR_(EVERGREEN)",
                        "score": 0.0
                    },
                    {
                        "source": "MILLÉSIMES_FPCI_ANNUELS",
                        "target": "FPCI",
                        "relation_type": "IS_A",
                        "description": "MILLÉSIMES_FPCI_ANNUELS IS_A FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "MILLÉSIMES_RAIF_(HORIZON)",
                        "target": "RAIF",
                        "relation_type": "IS_A",
                        "description": "MILLÉSIMES_RAIF_(HORIZON) IS_A RAIF",
                        "score": 0.0
                    },
                    {
                        "source": "ALTALIFE_2023_(FCPR)",
                        "target": "FCPR",
                        "relation_type": "IS_A",
                        "description": "ALTALIFE_2023_(FCPR) IS_A FCPR",
                        "score": 0.0
                    },
                    {
                        "source": "DISCOVERY_FCPR_(EVERGREEN)",
                        "target": "FCPR",
                        "relation_type": "IS_A",
                        "description": "DISCOVERY_FCPR_(EVERGREEN) IS_A FCPR",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "INSIGHT_PARTNERS",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN INSIGHT_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF",
                        "target": "SCA-SICAV-RAIF",
                        "relation_type": "IS_A",
                        "description": "ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF IS_A SCA-SICAV-RAIF",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "GENERAL_ATLANTIC",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN GENERAL_ATLANTIC",
                        "score": 0.0
                    },
                    {
                        "source": "GENERAL_ATLANTIC",
                        "target": "ODYSSEY_2021",
                        "relation_type": "INVESTS_IN",
                        "description": "GENERAL_ATLANTIC INVESTS_IN ODYSSEY_2021",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC",
                        "target": "TROPHÉE_PATRIMONIA",
                        "relation_type": "RECEIVED_AWARD",
                        "description": "ALTAROC RECEIVED_AWARD TROPHÉE_PATRIMONIA",
                        "score": 0.0
                    },
                    {
                        "source": "INSIGHT_PARTNERS",
                        "target": "ODYSSEY_2021",
                        "relation_type": "INVESTS_IN",
                        "description": "INSIGHT_PARTNERS INVESTS_IN ODYSSEY_2021",
                        "score": 0.0
                    },
                    {
                        "source": "MILLÉSIMES_FPCI_ANNUELS",
                        "target": "AMF",
                        "relation_type": "NOT_SUBJECT_TO",
                        "description": "MILLÉSIMES_FPCI_ANNUELS NOT_SUBJECT_TO AMF",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "FPCI",
                        "relation_type": "IS_A",
                        "description": "ODYSSEY IS_A FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "FPCI",
                        "target": "AMF",
                        "relation_type": "NOT_SUBJECT_TO",
                        "description": "FPCI NOT_SUBJECT_TO AMF",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "GENERAL_ATLANTIC",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH GENERAL_ATLANTIC",
                        "score": 0.0
                    },
                    {
                        "source": "ONEBRIEF",
                        "target": "ODYSSEY_2025",
                        "relation_type": "INVESTMENT",
                        "description": "ONEBRIEF INVESTMENT ODYSSEY_2025",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "INSIGHT_PARTNERS",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH INSIGHT_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "APAX_PARTNERS",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH APAX_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ONEBRIEF",
                        "target": "INSIGHT_PARTNERS",
                        "relation_type": "INVESTOR",
                        "description": "ONEBRIEF INVESTOR INSIGHT_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "NORDIC_CAPITAL",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH NORDIC_CAPITAL",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "SEVEN2",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH SEVEN2",
                        "score": 0.0
                    },
                    {
                        "source": "AMCS",
                        "target": "ODYSSEY_2021",
                        "relation_type": "INVESTS_IN",
                        "description": "AMCS INVESTS_IN ODYSSEY_2021",
                        "score": 0.0
                    },
                    {
                        "source": "QLIK",
                        "target": "ODYSSEY_2024",
                        "relation_type": "INVESTS_IN",
                        "description": "QLIK INVESTS_IN ODYSSEY_2024",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2022",
                        "target": "APAX",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2022 INVESTS_IN APAX",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2022",
                        "target": "BRIDGEPOINT",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2022 INVESTS_IN BRIDGEPOINT",
                        "score": 0.0
                    },
                    {
                        "source": "APAX_PARTNERS",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "APAX_PARTNERS INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "HG",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "HG INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "BRIDGEPOINT",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "BRIDGEPOINT INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "CRESSEY_&_COMPANY",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "CRESSEY_&_COMPANY INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "STG_(SOFTWARE_TECHNOLOGY_GROUP)",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "STG_(SOFTWARE_TECHNOLOGY_GROUP) INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "ACCEL-KKR",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "ACCEL-KKR INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2022",
                        "target": "ACCESS_GROUP",
                        "relation_type": "CO_INVESTED_WITH",
                        "description": "ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH ACCESS_GROUP",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "HG",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN HG",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2022",
                        "target": "AUTHORITY_BRANDS",
                        "relation_type": "CO_INVESTED_WITH",
                        "description": "ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH AUTHORITY_BRANDS",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2022",
                        "target": "IFS",
                        "relation_type": "CO_INVESTED_WITH",
                        "description": "ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH IFS",
                        "score": 0.0
                    },
                    {
                        "source": "APAX_PARTNERS",
                        "target": "AUTHORITY_BRANDS",
                        "relation_type": "INVESTED_IN",
                        "description": "APAX_PARTNERS INVESTED_IN AUTHORITY_BRANDS",
                        "score": 0.0
                    },
                    {
                        "source": "HG",
                        "target": "ACCESS_GROUP",
                        "relation_type": "INVESTED_IN",
                        "description": "HG INVESTED_IN ACCESS_GROUP",
                        "score": 0.0
                    },
                    {
                        "source": "HG",
                        "target": "IFS",
                        "relation_type": "INVESTED_IN",
                        "description": "HG INVESTED_IN IFS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "APAX",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN APAX",
                        "score": 0.0
                    },
                    {
                        "source": "SFDR_ARTICLE_8",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CLASSIFIED_AS",
                        "description": "SFDR_ARTICLE_8 CLASSIFIED_AS ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "AMCS",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CO_INVESTED_IN",
                        "description": "AMCS CO_INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "NORDIC_CAPITAL",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CO_INVESTED_IN",
                        "description": "NORDIC_CAPITAL CO_INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "SEVEN2",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CO_INVESTED_IN",
                        "description": "SEVEN2 CO_INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2024",
                        "target": "FPCI",
                        "relation_type": "USES",
                        "description": "ALTAROC_ODYSSEY_2024 USES FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2024",
                        "target": "THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS",
                        "relation_type": "INVESTS_IN",
                        "description": "ALTAROC_ODYSSEY_2024 INVESTS_IN THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2024",
                        "target": "QLIK",
                        "relation_type": "CO_INVESTS_WITH",
                        "description": "ALTAROC_ODYSSEY_2024 CO_INVESTS_WITH QLIK",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2025",
                        "target": "FPCI",
                        "relation_type": "USES",
                        "description": "ALTAROC_ODYSSEY_2025 USES FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC",
                        "target": "ODYSSEY_2023",
                        "relation_type": "OFFERS",
                        "description": "ALTAROC OFFERS ODYSSEY_2023",
                        "score": 0.0
                    },
                    {
                        "source": "INSIGHT_PARTNERS_XIII",
                        "target": "ALTAROC_ODYSSEY_2026",
                        "relation_type": "MANAGES",
                        "description": "INSIGHT_PARTNERS_XIII MANAGES ALTAROC_ODYSSEY_2026",
                        "score": 0.0
                    },
                    {
                        "source": "GBF_XIII",
                        "target": "ALTAROC_ODYSSEY_2026",
                        "relation_type": "MANAGES",
                        "description": "GBF_XIII MANAGES ALTAROC_ODYSSEY_2026",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2026",
                        "target": "INSIGHT_PARTNERS_XIII",
                        "relation_type": "INCLUDES",
                        "description": "ALTAROC_ODYSSEY_2026 INCLUDES INSIGHT_PARTNERS_XIII",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2026",
                        "target": "INFLEXION",
                        "relation_type": "INCLUDES",
                        "description": "ALTAROC_ODYSSEY_2026 INCLUDES INFLEXION",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "SEVEN2",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN SEVEN2",
                        "score": 0.0
                    }
                ]
            },
            "is_error": false,
            "created_at": "2026-06-29T13:47:36.747510+00:00",
            "updated_at": "2026-06-29T13:47:36.853301+00:00"
        }
    ]
}


------------

the sources ... 

---


curl 'http://localhost:8049/api/v1/chat/completions/stream' \
  -H 'Accept: */*' \
  -H 'Accept-Language: fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7' \
  -H 'Connection: keep-alive' \
  -H 'Origin: http://localhost:8048' \
  -H 'Referer: http://localhost:8048/' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-site' \
  -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36' \
  -H 'content-type: application/json' \
  -H 'sec-ch-ua: "Chromium";v="146", "Not-A.Brand";v="24", "Google Chrome";v="146"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "Linux"' \
  -H 'traceparent: 00-96968cd7da97105c4bc5a7bb3e4f8c0e-367a0423104d1792-01' \
  -H 'x-request-id: 12127f04-ebf0-4072-b3e2-825b0088d287' \
  -H 'x-tenant-id: b9d60c6e-c568-487a-a3b9-27318da1549d' \
  -H 'x-user-id: f640c273-9374-4c8f-8ad8-d7f582994a10' \
  -H 'x-workspace-id: 4382e7ab-6fb6-48b6-b49c-c3d1841dcc69' \
  --data-raw $'{"message":"c\'est quoi le produit odyssey 2026 altaroc ?","mode":"global","max_tokens":2048,"temperature":0.7,"top_k":10,"stream":true,"retrieval_only":true,"language":"fr"}'

  data: {"type":"conversation","conversation_id":"6216fa00-fa83-4bdf-b996-5d803381de88","user_message_id":"60239215-9810-46be-b2c9-51bb6f69168a"}

data: {"type":"context","sources":[{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6","score":4.005089,"snippet":"res** :\n  - Société d'investissement existant depuis plus de 15 ans\n  - Historique de performance 15 ans dans le 1er quartile\n  - Taux de perte historique < 2%\n  - Taille du fonds en levée > 1 Md€ et ","content":"res** :\n  - Société d'investissement existant depuis plus de 15 ans\n  - Historique de performance 15 ans dans le 1er quartile\n  - Taux de perte historique < 2%\n  - Taille du fonds en levée > 1 Md€ et AUM > 10 Mds€\n  - Qualité et cohésion des équipes\n  - Alignement d'intérêts (investissement significatif des gérants)\n  - Maîtrise de la data : bases propriétaires de performance\n\n---\n\n## GAMME 4 — ALTAROC GLOBAL EVERGREEN (Fonds Secondaire Multi-Pays)\n\n### Concept\n**Altaroc Global Evergreen SCA-SICAV-RAIF** est le fonds evergreen luxembourgeois d'Altaroc dédié au **marché secondaire du Private Equity**, commercialisé dans 9 pays européens depuis mars 2026.\n\n### Caractéristiques\n- **Format** : SCA-SICAV-RAIF (Luxembourg)\n- **Lancement** : Mars 2026\n- **Stratégie** : 100% Private Equity secondaire\n- **Souscriptions** : Périodes mensuelles de souscription et de rachat\n- **Minimum de souscription** : 100 000 € (variable selon pays et éligibilité)\n- **Pays de commercialisation** : Allemagne, Belgique, Espagne, France, Italie, Luxembourg, Pays-Bas, Portugal, Suisse\n- **Distribution** : Via banques privées, tiers-gérants, family offices, sociétés de gestion\n- **Marché secondaire** : A atteint un record de 240 milliards $ de levée en 2025 (+48% sur un an)\n- **Avantages** : Déploiement rapide du capital, réduction de la courbe en J, liquidité mensuelle\n\n---\n\n## PROGRAMME RE-UP — Stratégie Patrimoniale Long Terme\n\n### Concept\nLe **Programme Re-Up** est une discipline d'investissement (non un produit juridique distinct) qui consiste à s'engager dans **un nouveau millésime Odyssey chaque année**. Il s'inspire des pratiques des grands investisseurs institutionnels.\n\n### Fonctionnement\n- **Ticket de lancement** : À partir de 310 000 € (3 millésimes de 100 000 € sur 3 ans)\n- **Appels de fonds** : 10% de l'engagement par semestre pendant 5 ans par millésime\n- **Autofinancement** : À partir de la 6ème année, les distributions des anciens millésimes financent les appels des nouveaux\n- **Excédent** : À partir de la 10ème année, les distributions peuvent dépasser les appels\n\n### Projections illustratives (6 engagements annuels de 100 000 €)\n- Décaissements cumulés nets sur la période : ~310 000 €\n- À partir de l'an 7 : distributions potentielles ~170 000 €/an\n- Excédent potentiel brut : ~70 000 €/an\n- Net de flat tax (30%) : ~49 000 €/an nets de flux\n\n---\n\n## OFFRE SUR MESURE (≥ 1 000 000 €)\n\nPour les investisseurs disposant d'au moins **1 million d'euros** à allouer en Private Equity, Altaroc propose des solutions **sur mesure** :\n- Construction de portefeuilles personnalisés\n- Accès à des gérants spécifiques non disponibles dans les millésimes standards\n- Accompagnement dédié par les équipes d'investissement\n- Disponible via banques privées, family offices et gestionnaires de fortune\n- Structures adaptées à chaque juridiction et situation fiscale\n\n---\n\n## CANAUX DE DISTRIBUTION\n\n","reference_id":1,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":337,"end_line":400},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0","score":3.8888366,"snippet":"# ALTAROC — CATALOGUE COMPLET DES PRODUITS\n*Dernière mise à jour : Mai 2026 | Source : altaroc.pe*\n\n---\n\n## PRÉSENTATION GÉNÉRALE D'ALTAROC\n\n**Altaroc** (anciennement Amboise Partners, fondée en 1972)","content":"# ALTAROC — CATALOGUE COMPLET DES PRODUITS\n*Dernière mise à jour : Mai 2026 | Source : altaroc.pe*\n\n---\n\n## PRÉSENTATION GÉNÉRALE D'ALTAROC\n\n**Altaroc** (anciennement Amboise Partners, fondée en 1972) est une société de gestion française spécialisée dans le Private Equity à destination des investisseurs privés avertis. Fondée en 2021 sous sa forme actuelle par **Maurice Tchenio** (Président) et **Frédéric Stolar** (Managing Partner), Altaroc a pour mission de démocratiser l'accès au Private Equity institutionnel.\n\n### Chiffres clés (2025-2026)\n- **Plus de 1,4 milliard d'euros levés** en 4 ans\n- **Plus de 5 000 investisseurs privés**\n- **Présence dans 7+ pays européens**\n- **19+ gérants du 1er quartile mondial** sélectionnés\n- **Plus de 400 sociétés** en portefeuille (tous millésimes confondus)\n- **70 collaborateurs**\n- Élue **Plateforme de Private Equity de l'année 2024** (Gestion de Fortune)\n- Élue **Société la plus innovante 2023** (Gestion de Fortune)\n- **Trophée Patrimonia** : Meilleure relation auprès des CGP 2024\n\n### Équipe dirigeante\n| Nom | Poste |\n|-----|-------|\n| Maurice Tchenio | Président |\n| Frédéric Stolar | Managing Partner |\n| Louis Flamand | Chief Investment Officer |\n| Eric Sabia | Directeur Financier |\n| Claire Peyssard-Moses | Chief Reporting & ESG Officer |\n| Dimitri Bernard | Directeur d'investissements |\n\n---\n\n## ARCHITECTURE DE L'OFFRE\n\nAltaroc propose **4 gammes de produits** organisées selon le profil de l'investisseur et son horizon patrimonial :\n\n| Gamme | Produit | Type | Ticket min. | Horizon | Cible |\n|-------|---------|------|-------------|---------|-------|\n| **Odyssey** | Millésimes FPCI annuels | Fonds de fonds primaire | 100 000 € | 10 ans | Professionnels / Avertis |\n| **Odyssey** | Millésimes RAIF (Horizon) | Fonds de fonds primaire | Variable/juridiction | 10 ans | Internationaux |\n| **Discovery** | Altalife 2023 (FCPR) | Fonds de fonds primaire | Via assureur | 10 ans | Assurance vie / PER France |\n| **Discovery** | Discovery FCPR (Evergreen) | Secondaire evergreen | ~1 000 € (via AV) | 99 ans | Assurance vie / PER / PEA-PME |\n| **Global Evergreen** | Altaroc Global Evergreen SCA-SICAV-RAIF | Secondaire evergreen | 100 000 € | Perpétuel | Multi-pays |\n\n---\n\n## GAMME 1 — ODYSSEY (Millésimes FPCI Français)\n\n### Concept de base\nLes millésimes Odyssey sont des **Fonds Professionnels de Capital Investissement (FPCI)** non soumis à l'agrément AMF. Lancés chaque année depuis 2021, ils offrent un portefeuille clé en main de 5 à 7 fonds primaires de Private Equity internationaux (80%) + co-investissements (20%).\n\n","reference_id":2,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":1,"end_line":52},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5","score":2.0682943,"snippet":")\n- **Format** : RAIF\n- **Ouverture à la souscription** : 5 mai 2026\n- **Statut** : Ouvert\n- Stratégie alignée sur Odyssey 2026\n\n---\n\n## GAMME 3 — DISCOVERY (Assurance Vie & PER)\n\n### Concept\nLa gamme","content":")\n- **Format** : RAIF\n- **Ouverture à la souscription** : 5 mai 2026\n- **Statut** : Ouvert\n- Stratégie alignée sur Odyssey 2026\n\n---\n\n## GAMME 3 — DISCOVERY (Assurance Vie & PER)\n\n### Concept\nLa gamme Discovery permet l'accès au Private Equity Altaroc au sein des **contrats d'assurance vie français et PER**, avec une liquidité garantie par les assureurs. Elle comprend deux produits.\n\n---\n\n### ALTALIFE 2023 (FCPR — Millésimé, clôturé)\n- **Format** : FCPR agréé AMF\n- **Période de commercialisation** : jusqu'au 31/12/2024\n- **Statut** : Clôturé le 31/12/2024\n- **Éligibilité** : Assurance vie française + PER Assurance\n- **Durée** : 10 ans\n- **Minimum de souscription** : Montant défini par l'assureur\n- **Frais de gestion** : 2,5%\n- **Commission de performance** : 20% sur les co-investissements\n- **ESG** : Article 8 SFDR\n- **Liquidité** : Garantie à tout moment par l'assureur (VL bimensuelle)\n- **Collecte** : 180 M€ engagés\n- **Gérants sélectionnés** :\n  - **Hg Mercury 4** — Buy-Out Mid-Market logiciels, Europe (75 Mds$ AUM)\n  - **Five Arrows Principal Investment IV (FAPIIV)** — Buy-Out Mid-Market, logiciels/santé/services (Rothschild, 24 Mds€ AUM)\n- **Co-investissements réalisés** (4 sociétés) :\n  - **Visma** — Leader logiciels cloud (compta, paie, RH), Europe + Amérique Latine\n  - **Diapason** — Logiciels de gestion de trésorerie et risques financiers, France\n  - **Evoriel (ex-Nexity ADB)** — Gestion de copropriétés et baux locatifs, France\n  - **Howden** — Leader courtage en assurance, Europe\n\n---\n\n### ALTAROC DISCOVERY FCPR (Evergreen — EN COURS)\n- **Format** : FCPR agréé AMF (durée de vie 99 ans)\n- **ISIN** : FR001400Q544\n- **Statut** : Ouvert en permanence\n- **Éligibilité** : Assurance vie française, PER, PEA-PME, comptes-titres\n- **Minimum de souscription** : À partir de ~1 000 € (selon assureur / contrat)\n- **Valorisation** : Trimestrielle (avec VL estimatives bimensuelles)\n- **VL au 30/04/2026** : 112,98\n- **ESG** : Article 8 SFDR\n- **Stratégie** : 100% Private Equity **secondaire** (pour limiter la courbe en J)\n  - 70% LP-led secondaries (portefeuilles de fonds existants)\n  - 30% GP-led secondaries (restructurations de portefeuille)\n  - Uniquement matures et late secondaries\n- **Liquidité** :\n  - Pour l'assuré : garantie à tout moment par l'assureur à la dernière VL\n  - Pour l'assureur : pas de rachat pendant les 3 premières années ; ensuite gate de 6% par trimestre\n  - Trésorerie cible : 10% de l'actif net\n- **Frais de gestion** :\n  - Parts A : 2,00% / an\n  - Parts B (clean share) : 1,10% / an\n  - Parts P : 2,50% / an\n- **Commission de performance** : 10% de la croissance de la VL, payée tous les 3 ans\n- **Critères de sélection des gérants secondaires** :\n  - Société d'investissement existant depuis plus de 15 ans\n  - Historique de performance 15 ans dans le 1er quartile\n  - Taux de perte historique < 2%\n  - Taille du fonds en levée > 1 Md€ et AUM > 10 Mds€\n  - Qualité et cohésion des équipes\n  - Alignement d'intérêts (investissement significatif des gérants)\n  - Maîtrise de la data : bases propriétaires de performance\n\n---\n\n","reference_id":3,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":278,"end_line":337},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7","score":1.6751634,"snippet":" des solutions **sur mesure** :\n- Construction de portefeuilles personnalisés\n- Accès à des gérants spécifiques non disponibles dans les millésimes standards\n- Accompagnement dédié par les équipes d'i","content":" des solutions **sur mesure** :\n- Construction de portefeuilles personnalisés\n- Accès à des gérants spécifiques non disponibles dans les millésimes standards\n- Accompagnement dédié par les équipes d'investissement\n- Disponible via banques privées, family offices et gestionnaires de fortune\n- Structures adaptées à chaque juridiction et situation fiscale\n\n---\n\n## CANAUX DE DISTRIBUTION\n\nAltaroc distribue exclusivement **via des professionnels intermédiaires** :\n- **CGP** (Conseillers en Gestion de Patrimoine) — principal canal\n- **Banques privées**\n- **Family Offices**\n- **Tiers-gérants**\n- **Assureurs** (pour la gamme Discovery)\n\nAltaroc ne distribue **pas en direct** aux particuliers sans intermédiaire.\n\n---\n\n## STRUCTURE JURIDIQUE DES VÉHICULES UTILISÉS\n\n| Véhicule | Description | Agrément |\n|----------|-------------|----------|\n| **FPCI** | Fonds Professionnel de Capital Investissement | Non soumis à l'agrément AMF |\n| **SLP** | Société de Libre Partenariat | Flexible, investisseurs professionnels |\n| **FCPR** | Fonds Commun de Placement à Risques | Agréé AMF |\n| **RAIF** | Reserved Alternative Investment Fund (Luxembourg) | Réservé aux investisseurs qualifiés |\n| **SCA-SICAV-RAIF** | Société en Commandite par Actions / SICAV RAIF (Luxembourg) | Investisseurs qualifiés |\n| **ELTIF** | European Long-Term Investment Fund | Passeport européen |\n\n---\n\n## RÉCAPITULATIF DE TOUS LES PRODUITS\n\n| Produit | Millésime | Format | Collecte | Statut | Minimum |\n|---------|-----------|--------|----------|--------|---------|\n| Odyssey 2021 | 2021 | FPCI | 180 M€ | Clôturé | 100k€ |\n| Odyssey 2022 | 2022 | FPCI | 465 M€ | Clôturé | 100k€ |\n| Odyssey 2023 | 2023 | FPCI | 315 M€ | Clôturé | 100k€ |\n| Odyssey 2024 | 2024 | FPCI | 355 M€ | Clôturé | 100k€ |\n| Odyssey 2025 | 2025 | FPCI | 300 M€ | Fermé | 100k€ |\n| Odyssey 2026 | 2026 | SLP | En cours | Ouvert | 100k€ |\n| Horizon 2024 | 2024 | ELTIF | 8 M€ | Clôturé | Variable |\n| Horizon 2025 | 2025 | RAIF | 8 M€ | Fermé | Variable |\n| Horizon 2026 | 2026 | RAIF | En cours | Ouvert | Variable |\n| Altalife 2023 | 2023 | FCPR | 180 M€ | Clôturé | Via assureur |\n| Discovery FCPR | Evergreen | FCPR | En cours | Ouvert | ~1 000 € |\n| Altaroc Global Evergreen | Evergreen | SCA-SICAV-RAIF | En cours | Ouvert | 100k€ |\n\n*Total levé (tous produits confondus) : > 1,4 Md€*\n\n---\n\n*Note : Les performances passées ne préjugent pas des performances futures. L'investissement en Private Equity comporte des risques de perte en capital et d'illiquidité. Ces informations sont à caractère informatif et ne constituent pas un conseil en investissement.*\n","reference_id":4,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":400,"end_line":400},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4","score":1.3554758,"snippet":" AUM, 380+ personnes)\n  - **Hg Saturn 4** — LBO Large Cap logiciels, Europe + US (même gérant Hg)\n  - **Insight XIII + GBF XIII** — Growth Equity logiciels à forte croissance (fondé 1995, 80 Mds$ AUM,","content":" AUM, 380+ personnes)\n  - **Hg Saturn 4** — LBO Large Cap logiciels, Europe + US (même gérant Hg)\n  - **Insight XIII + GBF XIII** — Growth Equity logiciels à forte croissance (fondé 1995, 80 Mds$ AUM, 154+ personnes)\n- **Allocation sectorielle cible** : 50-60% logiciels/tech, 10-20% santé, ~20% services, ~10% conso\n- **Société exemple** : AMCS (solutions logicielles industrie, Irlande)\n- **Sociétés émergentes** : Cleerly (IA cardiovasculaire), Onebrief (logiciels défense, valorisé 1,1 Md$), Specter Ops (cybersécurité)\n\n---\n\n### ALTAROC ODYSSEY 2026 (6ème Millésime — EN COURS)\n- **Format** : SLP (Société de Libre Partenariat)\n- **Ouverture à la souscription** : 5 mai 2026\n- **Statut** : Ouvert\n- **Minimum de souscription** : 100 000 €, appelés à 10% par semestre sur 5 ans\n- **ESG** : Article 8 SFDR\n- **Univers de sélection** : 7 fonds cibles (2 déjà sélectionnés + 5 en analyse avancée)\n- **Gérants déjà engagés** :\n  - **Insight Partners XIII** — Growth Equity, logiciels en forte croissance\n  - **Hg Saturn 4** — LBO Large Cap logiciels, Europe + US\n- **Gérants en univers de sélection** : Inflexion, Great Hill Partners, Nordic Capital XII, New Mountain Strategic Equity Fund II, Hg Mercury 5, K1 Investment Management (K6)\n- **Allocation cible** : 50-60% tech/logiciels, 10-20% santé, ~20% services, ~10% conso\n\n---\n\n## GAMME 2 — HORIZON (Millésimes RAIF Internationaux)\n\n### Concept\nLes millésimes **Horizon** sont l'équivalent international des millésimes Odyssey. Structurés en **RAIF (Reserved Alternative Investment Fund)** luxembourgeois, ils permettent l'accès à la même stratégie pour des investisseurs résidents dans des pays hors France (Portugal, Espagne, Italie, Suisse, Belgique, Pays-Bas, Allemagne, Singapour, UK).\n\n### Même stratégie qu'Odyssey\n- 80% fonds primaires + 20% co-investissements\n- Mêmes gérants sélectionnés\n- Même TRI cible (13% par an), même multiple cible (1,7x)\n- Frais identiques (2,5% à 1,65% dégressifs)\n- Article 8 SFDR\n\n---\n\n### ALTAROC HORIZON 2024\n- **Collecte** : 8 M€\n- **Format** : ELTIF (European Long-Term Investment Fund)\n- **Statut** : Fermé\n- **Stratégie** : Identique à Odyssey 2024\n\n---\n\n### ALTAROC HORIZON 2025\n- **Taille cible** : 50 M€\n- **Format** : RAIF luxembourgeois\n- **Période de commercialisation** : 15/01/2026 – 31/03/2026\n- **Statut** : Fermé\n- **Minimum de souscription** : Variable selon juridiction\n- **Durée** : 10 ans\n- **ESG** : Article 8 SFDR\n- **Gérants** : Identiques à Odyssey 2025 (K1, Nordic Capital XII, New Mountain, Hg Mercury 5, Hg Saturn 4, Insight XIII + GBF XIII, Inflexion Buyout Fund VII)\n\n---\n\n### ALTAROC HORIZON 2026 (EN COURS)\n- **Format** : RAIF\n- **Ouverture à la souscription** : 5 mai 2026\n- **Statut** : Ouvert\n- Stratégie alignée sur Odyssey 2026\n\n---\n\n## GAMME 3 — DISCOVERY (Assurance Vie & PER)\n\n### Concept\nLa gamme Discovery permet l'accès au Private Equity Altaroc au sein des **contrats d'assurance vie français et PER**, avec une liquidité garantie par les assureurs. Elle comprend deux produits.\n\n---\n\n","reference_id":5,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":202,"end_line":278},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1","score":0.87912405,"snippet":" | Perpétuel | Multi-pays |\n\n---\n\n## GAMME 1 — ODYSSEY (Millésimes FPCI Français)\n\n### Concept de base\nLes millésimes Odyssey sont des **Fonds Professionnels de Capital Investissement (FPCI)** non sou","content":" | Perpétuel | Multi-pays |\n\n---\n\n## GAMME 1 — ODYSSEY (Millésimes FPCI Français)\n\n### Concept de base\nLes millésimes Odyssey sont des **Fonds Professionnels de Capital Investissement (FPCI)** non soumis à l'agrément AMF. Lancés chaque année depuis 2021, ils offrent un portefeuille clé en main de 5 à 7 fonds primaires de Private Equity internationaux (80%) + co-investissements (20%).\n\n### Structure d'investissement\n- **80%** investi dans 5 à 7 fonds gérés par des gérants mondiaux de premier plan\n- **20%** investi en co-investissements aux côtés des mêmes gérants (sans frais de gestion supplémentaires, sans carried interest sur la part fonds)\n- **Stratégies** : exclusivement **Growth Equity** et **Buy-Out** (les 2 segments les moins volatils et plus rentables du PE)\n- **Secteurs** : Technologie/Logiciels (50-60%), Santé (10-20%), Services B2B (~20%), Consommation (~10%)\n- **Géographies** : Europe (~45%) + Amérique du Nord (~45%) + Reste du monde (~10%)\n\n### Structure des appels de fonds\n- Appels à **date fixe et quantum fixe** : 10% de l'engagement tous les semestres (fin mars et fin octobre)\n- Capital entièrement appelé sur **5 ans**\n- Engagement minimum de 100 000 € = ~20 000 € par an pendant 5 ans\n\n### Frais de gestion (dégressifs selon le montant)\n| Montant souscrit | Frais de gestion annuels |\n|-----------------|--------------------------|\n| Tranche standard | 2,5% |\n| Tranches intermédiaires | Dégressifs |\n| Tranche haute | 1,65% |\n- **Commission de performance (carried interest)** : 20% exclusivement sur les co-investissements (pas sur la part fonds)\n\n### Objectifs de performance (non garantis)\n- **TRI cible** : 13% par an avant fiscalité\n- **Multiple cible** : 1,7x l'engagement avant fiscalité\n- **Track record historique** des gérants sélectionnés : TRI moyen net de 21,8% / multiple de 2,26x (millésimes 1999-2018)\n\n---\n\n### ALTAROC ODYSSEY 2021 (1er Millésime)\n- **Collecte** : 180 M€\n- **Format** : FPCI\n- **Période de commercialisation** : 2021\n- **Statut** : Clôturé\n- **Gérants sélectionnés** :\n  - **General Atlantic** — Growth Equity, empreinte mondiale\n  - **Insight Partners** — Growth, logiciels en forte croissance\n  - **Apax Partners** — Buy-Out global, multi-secteurs\n  - **Nordic Capital** — Buy-Out pan-européen (santé, tech)\n  - **Seven2** — Buy-Out mid-market européen\n- **Engagements fonds** : 144 M€ dans 5 fonds internationaux\n- **Co-investissements** : 5 sociétés (dont AMCS - dernier co-invest signé en 2024)\n- **ESG** : Article 8 SFDR\n\n---\n\n","reference_id":6,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":52,"end_line":105},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3","score":0.46388084,"snippet":"xpertise logiciels\n  - **New Mountain Capital VII** — Buy-Out nord-américain (tech, santé, services)\n  - **Inflexion Partnership Capital III** — Buy-Out minoritaire, Europe du Nord\n  - **Main Capital ","content":"xpertise logiciels\n  - **New Mountain Capital VII** — Buy-Out nord-américain (tech, santé, services)\n  - **Inflexion Partnership Capital III** — Buy-Out minoritaire, Europe du Nord\n  - **Main Capital VIII & Foundation II** — Buy-Out logiciels, Europe du Nord et US\n- **Part fonds engagée** : 264 M€ dans 5 gérants\n- **Co-investissements** : 66 M€ cibles, 5 sociétés réalisées\n\n---\n\n### ALTAROC ODYSSEY 2024 (4ème Millésime)\n- **Collecte** : 355 M€\n- **Format** : FPCI\n- **Période de commercialisation** : 04/04/2024 – 31/03/2025\n- **Statut** : Clôturé\n- **Minimum de souscription** : 100 000 €\n- **Durée** : 10 ans\n- **Investissement des fondateurs** : 33 M€\n- **ESG** : Article 8 SFDR\n- **TRI cible** : 13% par an\n- **Multiple cible** : 1,7x\n- **Gérants sélectionnés (5 fonds, 4 gérants)** :\n  - **Vitruvian Investment Partners V + Genesis** (70M€ + 40M€) — Growth Buy-Out, global à dominante européenne (fondé 2006, 16 Mds€ AUM, 190 personnes, 8 bureaux)\n  - **Thoma Bravo Discover V + XVI + Genesis** (45M€ + 40M€ + 40M€) — Buy-Out Large Cap / Upper Mid-Market logiciels, Amérique du Nord (fondé 1998, 166 Mds$ AUM, 220 personnes)\n  - **Summit Partners XII + Genesis** (70M€ + 40M€) — Growth Equity, Amérique du Nord (fondé 1984, 37 Mds$ AUM, 239 personnes)\n  - **Bridgepoint Europe VII + DC V + Genesis** (80M€ + 35M€ + 40M€) — Buy-Out Lower to Mid-Market, Europe (fondé 1984, 41 Mds€ AUM, 560+ personnes)\n- **Part fonds engagée** : 260 M€ dans 4 gérants\n- **Co-investissement réalisé** : **Qlik** (leader business intelligence/data analytics, US) — aux côtés de Thoma Bravo\n- **Éligibilité** : Assurance vie luxembourgeoise + 10 pays (Luxembourg, Portugal, Espagne, Italie, Suisse, Belgique, Pays-Bas, Allemagne, Singapour, UK)\n\n---\n\n### ALTAROC ODYSSEY 2025 (5ème Millésime)\n- **Collecte** : 300 M€ (cible 310 M€)\n- **Format** : FPCI\n- **Période de commercialisation** : 04/04/2025 – 31/03/2026\n- **Statut** : Fermé à la souscription\n- **Minimum de souscription** : 100 000 €\n- **Durée** : 10 ans\n- **Investissement des fondateurs** : 33 M€\n- **ESG** : Article 8 SFDR\n- **TRI cible** : 13% par an\n- **Multiple cible** : 1,7x\n- **Gérants sélectionnés (6 fonds primaires)** :\n  - **K1 Investment Management (K6)** — Buy-Out logiciels, Amérique du Nord (fondé 2010, 20 Mds$ AUM, 100+ personnes)\n  - **Nordic Capital XII** — Buy-Out santé & tech, Europe (fondé 1989, 31 Mds$ AUM, 230+ personnes)\n  - **New Mountain Strategic Equity Fund II** — Growth / Buy-Out, US (fondé 1999, 55 Mds$ AUM, 260+ personnes)\n  - **Hg Mercury 5** — Buy-Out Mid-Market logiciels, Europe (fondé 1990, 75 Mds$ AUM, 380+ personnes)\n  - **Hg Saturn 4** — LBO Large Cap logiciels, Europe + US (même gérant Hg)\n  - **Insight XIII + GBF XIII** — Growth Equity logiciels à forte croissance (fondé 1995, 80 Mds$ AUM, 154+ personnes)\n- **Allocation sectorielle cible** : 50-60% logiciels/tech, 10-20% santé, ~20% services, ~10% conso\n- **Société exemple** : AMCS (solutions logicielles industrie, Irlande)\n","reference_id":7,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":152,"end_line":202},{"source_type":"chunk","id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2","score":0.17031896,"snippet":"ers** — Growth, logiciels en forte croissance\n  - **Apax Partners** — Buy-Out global, multi-secteurs\n  - **Nordic Capital** — Buy-Out pan-européen (santé, tech)\n  - **Seven2** — Buy-Out mid-market eur","content":"ers** — Growth, logiciels en forte croissance\n  - **Apax Partners** — Buy-Out global, multi-secteurs\n  - **Nordic Capital** — Buy-Out pan-européen (santé, tech)\n  - **Seven2** — Buy-Out mid-market européen\n- **Engagements fonds** : 144 M€ dans 5 fonds internationaux\n- **Co-investissements** : 5 sociétés (dont AMCS - dernier co-invest signé en 2024)\n- **ESG** : Article 8 SFDR\n\n---\n\n### ALTAROC ODYSSEY 2022 (2ème Millésime)\n- **Collecte** : 465 M€ (record de la gamme)\n- **Format** : FPCI\n- **Période de commercialisation** : 01/02/2022 – 30/03/2023\n- **Statut** : Clôturé\n- **Investissement des fondateurs** : 40 M€\n- **ESG** : Article 8 SFDR\n- **Gérants sélectionnés (7 fonds, 6 gérants)** :\n  - **Hg** — Buy-Out logiciels, Europe (Hg Saturn 3 + Hg Genesis 10)\n  - **Apax** — Buy-Out global (Apax X LP + Apax XI LP)\n  - **Bridgepoint** — Buy-Out pan-européen (Bridgepoint Europe VII)\n  - **Cressey & Company** — Buy-Out santé, États-Unis (Cressey Fund VII)\n  - **STG (Software Technology Group)** — Buy-Out logiciels valeur, US (STG Fund VII)\n  - **Accel-KKR** — Buy-Out logiciels growth, US (Accel-KKR Buy-Out VII)\n- **Part fonds engagée** : 372 M€ (7 fonds)\n- **Co-investissements** (7 opérations, 93 M€ cibles) :\n  - The Access Group (logiciels ETI, UK) — avec Hg\n  - Adevinta (petites annonces en ligne, Norvège) — avec Apax\n  - Authority Brands (franchises services à domicile, US) — avec Apax\n  - Howden (courtage assurance, Europe) — avec Hg\n  - IFS (ERP industriel, Europe) — avec Hg\n  - Evoriel/ex-Nexity ADB (gestion immobilière, France) — avec Bridgepoint\n  - ThoughtWorks (transformation digitale, US) — avec General Atlantic\n\n---\n\n### ALTAROC ODYSSEY 2023 (3ème Millésime)\n- **Collecte** : 315 M€\n- **Format** : FPCI\n- **Statut** : Clôturé\n- **ESG** : Article 8 SFDR\n- **Gérants sélectionnés (5 gérants, 6 fonds)** :\n  - **CVC Capital Partners IX** — Buy-Out Large Cap, Europe/monde (logiciels, santé, services, conso)\n  - **TA Associates XV** — Growth/Buy-Out global, forte expertise logiciels\n  - **New Mountain Capital VII** — Buy-Out nord-américain (tech, santé, services)\n  - **Inflexion Partnership Capital III** — Buy-Out minoritaire, Europe du Nord\n  - **Main Capital VIII & Foundation II** — Buy-Out logiciels, Europe du Nord et US\n- **Part fonds engagée** : 264 M€ dans 5 gérants\n- **Co-investissements** : 66 M€ cibles, 5 sociétés réalisées\n\n---\n\n","reference_id":8,"document_id":"cdfd2960-2444-48e5-8e3b-8b6b1540560a","file_path":"altaroc_produits.md","start_line":105,"end_line":152},{"source_type":"entity","id":"ODYSSEY_2021","score":0.0,"snippet":"FPCI investment product from 2021, closed with 180M€ raised","content":"FPCI investment product from 2021, closed with 180M€ raised","reference_id":9,"entity_type":"PRODUCT","degree":71,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"INSIGHT_PARTNERS","score":0.0,"snippet":"Growth equity investment firm focused on software","content":"Growth equity investment firm focused on software","reference_id":10,"entity_type":"ORGANIZATION","degree":38,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ALTAROC","score":0.0,"snippet":"Société française de gestion spécialisée dans le Private Equity pour investisseurs privés avertis.","content":"Société française de gestion spécialisée dans le Private Equity pour investisseurs privés avertis.","reference_id":11,"entity_type":"ORGANIZATION","degree":30,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2026","score":0.0,"snippet":"6th vintage of Altaroc's Odyssey fund series","content":"6th vintage of Altaroc's Odyssey fund series","reference_id":12,"entity_type":"EVENT","degree":25,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"GENERAL_ATLANTIC","score":0.0,"snippet":"Global growth equity investment firm","content":"Global growth equity investment firm","reference_id":13,"entity_type":"ORGANIZATION","degree":25,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ODYSSEY","score":0.0,"snippet":"Gamme de produits d'Altaroc basée sur des millésimes FPCI annuels.","content":"Gamme de produits d'Altaroc basée sur des millésimes FPCI annuels.","reference_id":14,"entity_type":"PRODUCT","degree":20,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"ODYSSEY_2022","score":0.0,"snippet":"FPCI investment product from 2022, closed with 465M€ raised","content":"FPCI investment product from 2022, closed with 465M€ raised","reference_id":15,"entity_type":"PRODUCT","degree":19,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"ALTAROC_DISCOVERY_FCPR","score":0.0,"snippet":"FCPR Evergreen ouvert en permanence accessible via plusieurs supports d'investissement","content":"FCPR Evergreen ouvert en permanence accessible via plusieurs supports d'investissement","reference_id":16,"entity_type":"PRODUCT","degree":16,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"]},{"source_type":"entity","id":"ODYSSEY_2023","score":0.0,"snippet":"FPCI investment product from 2023, closed with 315M€ raised","content":"FPCI investment product from 2023, closed with 315M€ raised","reference_id":17,"entity_type":"PRODUCT","degree":14,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"ALTALIFE_2023","score":0.0,"snippet":"FCPR millésimé et clôturé accessible via assurance vie française et PER","content":"FCPR millésimé et clôturé accessible via assurance vie française et PER","reference_id":18,"entity_type":"PRODUCT","degree":13,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"]},{"source_type":"entity","id":"ALTAROC_HORIZON_2025","score":0.0,"snippet":"Closed RAIF fund targeting 50M€","content":"Closed RAIF fund targeting 50M€","reference_id":19,"entity_type":"EVENT","degree":13,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2025","score":0.0,"snippet":"5th vintage of ALTAROC ODYSSEY investment fund","content":"5th vintage of ALTAROC ODYSSEY investment fund","reference_id":20,"entity_type":"EVENT","degree":8,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"NORDIC_CAPITAL","score":0.0,"snippet":"Pan-European buyout firm focused on healthcare and technology","content":"Pan-European buyout firm focused on healthcare and technology","reference_id":21,"entity_type":"ORGANIZATION","degree":8,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"PROGRAMME_RE-UP","score":0.0,"snippet":"Stratégie d'investissement patrimonial long terme d'Altaroc","content":"Stratégie d'investissement patrimonial long terme d'Altaroc","reference_id":22,"entity_type":"CONCEPT","degree":8,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6"]},{"source_type":"entity","id":"HG","score":0.0,"snippet":"Buy-out firm focused on software in Europe","content":"Buy-out firm focused on software in Europe","reference_id":23,"entity_type":"ORGANIZATION","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"Second vintage of Altaroc Odyssey fund with 465M€ raised","content":"Second vintage of Altaroc Odyssey fund with 465M€ raised","reference_id":24,"entity_type":"EVENT","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ALTAROC_ODYSSEY_2024","score":0.0,"snippet":"4th vintage of ALTAROC ODYSSEY investment fund","content":"4th vintage of ALTAROC ODYSSEY investment fund","reference_id":25,"entity_type":"EVENT","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"ODYSSEY_2024","score":0.0,"snippet":"FPCI investment product from 2024, closed with 355M€ raised","content":"FPCI investment product from 2024, closed with 355M€ raised","reference_id":26,"entity_type":"PRODUCT","degree":7,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"AMF","score":0.0,"snippet":"Autorité des Marchés Financiers, régulateur français des marchés financiers.","content":"Autorité des Marchés Financiers, régulateur français des marchés financiers.","reference_id":27,"entity_type":"ORGANIZATION","degree":6,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"APAX","score":0.0,"snippet":"Global buy-out firm","content":"Global buy-out firm","reference_id":28,"entity_type":"ORGANIZATION","degree":6,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF","score":0.0,"snippet":"Fonds secondaire evergreen multi-pays de la gamme Global Evergreen.","content":"Fonds secondaire evergreen multi-pays de la gamme Global Evergreen.","reference_id":29,"entity_type":"PRODUCT","degree":6,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"BRIDGEPOINT","score":0.0,"snippet":"Pan-European buy-out firm","content":"Pan-European buy-out firm","reference_id":30,"entity_type":"ORGANIZATION","degree":5,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"AMCS","score":0.0,"snippet":"Company involved in co-investment (last signed in 2024)","content":"Company involved in co-investment (last signed in 2024)","reference_id":31,"entity_type":"ORGANIZATION","degree":4,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ODYSSEY_2026","score":0.0,"snippet":"Strategic event or initiative referenced in the text","content":"Strategic event or initiative referenced in the text","reference_id":32,"entity_type":"EVENT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"]},{"source_type":"entity","id":"ALTAROC_HORIZON_2026","score":0.0,"snippet":"Open RAIF fund with subscription opening on 5 May 2026","content":"Open RAIF fund with subscription opening on 5 May 2026","reference_id":33,"entity_type":"EVENT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"ALTAROC_HORIZON_2024","score":0.0,"snippet":"Closed ELTIF fund with 8M€ raised","content":"Closed ELTIF fund with 8M€ raised","reference_id":34,"entity_type":"EVENT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"XPERTISE_LOGICIELS","score":0.0,"snippet":"Firm managing investment funds","content":"Firm managing investment funds","reference_id":35,"entity_type":"ORGANIZATION","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"DISCOVERY","score":0.0,"snippet":"Gamme de produits d'Altaroc incluant des fonds FCPR et secondaires evergreen.","content":"Gamme de produits d'Altaroc incluant des fonds FCPR et secondaires evergreen.","reference_id":36,"entity_type":"PRODUCT","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"APAX_PARTNERS","score":0.0,"snippet":"Global buyout investment firm","content":"Global buyout investment firm","reference_id":37,"entity_type":"ORGANIZATION","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"SEVEN2","score":0.0,"snippet":"European mid-market buyout firm","content":"European mid-market buyout firm","reference_id":38,"entity_type":"ORGANIZATION","degree":3,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"]},{"source_type":"entity","id":"ALTAROC_DISCOVERY","score":0.0,"snippet":"Private Equity access product within French life insurance and PER contracts","content":"Private Equity access product within French life insurance and PER contracts","reference_id":39,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"ACCEL-KKR","score":0.0,"snippet":"US-based buy-out firm focused on software growth","content":"US-based buy-out firm focused on software growth","reference_id":40,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ONEBRIEF","score":0.0,"snippet":"Defense software company valued at $1.1B","content":"Defense software company valued at $1.1B","reference_id":41,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"QLIK","score":0.0,"snippet":"Business intelligence/data analytics company","content":"Business intelligence/data analytics company","reference_id":42,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"CLEERLY","score":0.0,"snippet":"AI cardiovascular company","content":"AI cardiovascular company","reference_id":43,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"AUTHORITY_BRANDS","score":0.0,"snippet":"US-based home services franchises company","content":"US-based home services franchises company","reference_id":44,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"SFDR_ARTICLE_8","score":0.0,"snippet":"Sustainable Finance Disclosure Regulation classification for funds promoting environmental or social characteristics","content":"Sustainable Finance Disclosure Regulation classification for funds promoting environmental or social characteristics","reference_id":45,"entity_type":"LAW_NODE","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"HORIZON_2024","score":0.0,"snippet":"ELTIF investment product from 2024, closed with 8M€ raised","content":"ELTIF investment product from 2024, closed with 8M€ raised","reference_id":46,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"MILLÉSIMES_RAIF_(HORIZON)","score":0.0,"snippet":"Fonds de fonds primaires proposés dans la gamme Odyssey pour investisseurs internationaux.","content":"Fonds de fonds primaires proposés dans la gamme Odyssey pour investisseurs internationaux.","reference_id":47,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"ODYSSEY_2025","score":0.0,"snippet":"FPCI investment product from 2025, closed with 300M€ raised","content":"FPCI investment product from 2025, closed with 300M€ raised","reference_id":48,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"HORIZON_2025","score":0.0,"snippet":"RAIF investment product from 2025, closed with 8M€ raised","content":"RAIF investment product from 2025, closed with 8M€ raised","reference_id":49,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"MILLÉSIMES_FPCI_ANNUELS","score":0.0,"snippet":"Fonds de fonds primaires proposés dans la gamme Odyssey.","content":"Fonds de fonds primaires proposés dans la gamme Odyssey.","reference_id":50,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"HORIZON_2026","score":0.0,"snippet":"RAIF investment product from 2026, currently open","content":"RAIF investment product from 2026, currently open","reference_id":51,"entity_type":"PRODUCT","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"]},{"source_type":"entity","id":"SPECTER_OPS","score":0.0,"snippet":"Cybersecurity company","content":"Cybersecurity company","reference_id":52,"entity_type":"ORGANIZATION","degree":2,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"STG_(SOFTWARE_TECHNOLOGY_GROUP)","score":0.0,"snippet":"US-based buy-out firm focused on software value","content":"US-based buy-out firm focused on software value","reference_id":53,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"CRESSEY_&_COMPANY","score":0.0,"snippet":"US-based buy-out firm focused on healthcare","content":"US-based buy-out firm focused on healthcare","reference_id":54,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"SCA-SICAV-RAIF","score":0.0,"snippet":"Structure juridique d'investissement (Société en Commandite par Actions, Société d'Investissement à Capital Variable, RAIF).","content":"Structure juridique d'investissement (Société en Commandite par Actions, Société d'Investissement à Capital Variable, RAIF).","reference_id":55,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"INFLEXION","score":0.0,"snippet":"Private equity firm in selection universe","content":"Private equity firm in selection universe","reference_id":56,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"FCPR","score":0.0,"snippet":"Fonds Communs de Placement à Risques, structure d'investissement en Private Equity.","content":"Fonds Communs de Placement à Risques, structure d'investissement en Private Equity.","reference_id":57,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"GBF_XIII","score":0.0,"snippet":"Growth Equity fund founded in 1995 with $80B AUM","content":"Growth Equity fund founded in 1995 with $80B AUM","reference_id":58,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"RAIF","score":0.0,"snippet":"Reserved Alternative Investment Fund, structure d'investissement luxembourgeoise.","content":"Reserved Alternative Investment Fund, structure d'investissement luxembourgeoise.","reference_id":59,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"INSIGHT_PARTNERS_XIII","score":0.0,"snippet":"Growth Equity fund focused on high-growth software","content":"Growth Equity fund focused on high-growth software","reference_id":60,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"]},{"source_type":"entity","id":"TROPHÉE_PATRIMONIA","score":0.0,"snippet":"Prix récompensant la meilleure relation auprès des CGP en 2024.","content":"Prix récompensant la meilleure relation auprès des CGP en 2024.","reference_id":61,"entity_type":"EVENT","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"IFS","score":0.0,"snippet":"European industrial ERP company","content":"European industrial ERP company","reference_id":62,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"ACCESS_GROUP","score":0.0,"snippet":"UK-based mid-market software company","content":"UK-based mid-market software company","reference_id":63,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"entity","id":"FPCI","score":0.0,"snippet":"Fonds Professionnels de Capital Investissement, structure d'investissement en Private Equity.","content":"Fonds Professionnels de Capital Investissement, structure d'investissement en Private Equity.","reference_id":64,"entity_type":"TECHNOLOGY","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"DISCOVERY_FCPR_(EVERGREEN)","score":0.0,"snippet":"Fonds secondaire evergreen de la gamme Discovery accessible via assurance vie/PER/PEA-PME.","content":"Fonds secondaire evergreen de la gamme Discovery accessible via assurance vie/PER/PEA-PME.","reference_id":65,"entity_type":"PRODUCT","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS","score":0.0,"snippet":"Buy-out fund for large cap/upper mid-market software in North America","content":"Buy-out fund for large cap/upper mid-market software in North America","reference_id":66,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"]},{"source_type":"entity","id":"ALTALIFE_2023_(FCPR)","score":0.0,"snippet":"Fonds de fonds primaire de la gamme Discovery pour assurance vie/PER en France.","content":"Fonds de fonds primaire de la gamme Discovery pour assurance vie/PER en France.","reference_id":67,"entity_type":"PRODUCT","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"]},{"source_type":"entity","id":"TA_ASSOCIATES_XV","score":0.0,"snippet":"Global growth/buy-out firm with strong software expertise","content":"Global growth/buy-out firm with strong software expertise","reference_id":68,"entity_type":"ORGANIZATION","degree":1,"source_chunk_ids":["cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"]},{"source_type":"relationship","id":"ALTAROC->ODYSSEY","score":0.0,"snippet":"ALTAROC OFFERS ODYSSEY","reference_id":69},{"source_type":"relationship","id":"ALTAROC->DISCOVERY","score":0.0,"snippet":"ALTAROC OFFERS DISCOVERY","reference_id":70},{"source_type":"relationship","id":"ODYSSEY_2022->HG","score":0.0,"snippet":"ODYSSEY_2022 INVESTS_IN HG","reference_id":71},{"source_type":"relationship","id":"ODYSSEY_2021->NORDIC_CAPITAL","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN NORDIC_CAPITAL","reference_id":72},{"source_type":"relationship","id":"ODYSSEY->MILLÉSIMES_FPCI_ANNUELS","score":0.0,"snippet":"ODYSSEY INCLUDES MILLÉSIMES_FPCI_ANNUELS","reference_id":73},{"source_type":"relationship","id":"ODYSSEY->MILLÉSIMES_RAIF_(HORIZON)","score":0.0,"snippet":"ODYSSEY INCLUDES MILLÉSIMES_RAIF_(HORIZON)","reference_id":74},{"source_type":"relationship","id":"DISCOVERY->ALTALIFE_2023_(FCPR)","score":0.0,"snippet":"DISCOVERY INCLUDES ALTALIFE_2023_(FCPR)","reference_id":75},{"source_type":"relationship","id":"DISCOVERY->DISCOVERY_FCPR_(EVERGREEN)","score":0.0,"snippet":"DISCOVERY INCLUDES DISCOVERY_FCPR_(EVERGREEN)","reference_id":76},{"source_type":"relationship","id":"MILLÉSIMES_FPCI_ANNUELS->FPCI","score":0.0,"snippet":"MILLÉSIMES_FPCI_ANNUELS IS_A FPCI","reference_id":77},{"source_type":"relationship","id":"MILLÉSIMES_RAIF_(HORIZON)->RAIF","score":0.0,"snippet":"MILLÉSIMES_RAIF_(HORIZON) IS_A RAIF","reference_id":78},{"source_type":"relationship","id":"ALTALIFE_2023_(FCPR)->FCPR","score":0.0,"snippet":"ALTALIFE_2023_(FCPR) IS_A FCPR","reference_id":79},{"source_type":"relationship","id":"DISCOVERY_FCPR_(EVERGREEN)->FCPR","score":0.0,"snippet":"DISCOVERY_FCPR_(EVERGREEN) IS_A FCPR","reference_id":80},{"source_type":"relationship","id":"ODYSSEY_2021->INSIGHT_PARTNERS","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN INSIGHT_PARTNERS","reference_id":81},{"source_type":"relationship","id":"ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF->SCA-SICAV-RAIF","score":0.0,"snippet":"ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF IS_A SCA-SICAV-RAIF","reference_id":82},{"source_type":"relationship","id":"ODYSSEY_2021->GENERAL_ATLANTIC","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN GENERAL_ATLANTIC","reference_id":83},{"source_type":"relationship","id":"GENERAL_ATLANTIC->ODYSSEY_2021","score":0.0,"snippet":"GENERAL_ATLANTIC INVESTS_IN ODYSSEY_2021","reference_id":84},{"source_type":"relationship","id":"ALTAROC->TROPHÉE_PATRIMONIA","score":0.0,"snippet":"ALTAROC RECEIVED_AWARD TROPHÉE_PATRIMONIA","reference_id":85},{"source_type":"relationship","id":"INSIGHT_PARTNERS->ODYSSEY_2021","score":0.0,"snippet":"INSIGHT_PARTNERS INVESTS_IN ODYSSEY_2021","reference_id":86},{"source_type":"relationship","id":"MILLÉSIMES_FPCI_ANNUELS->AMF","score":0.0,"snippet":"MILLÉSIMES_FPCI_ANNUELS NOT_SUBJECT_TO AMF","reference_id":87},{"source_type":"relationship","id":"ODYSSEY->FPCI","score":0.0,"snippet":"ODYSSEY IS_A FPCI","reference_id":88},{"source_type":"relationship","id":"FPCI->AMF","score":0.0,"snippet":"FPCI NOT_SUBJECT_TO AMF","reference_id":89},{"source_type":"relationship","id":"ODYSSEY->GENERAL_ATLANTIC","score":0.0,"snippet":"ODYSSEY INVESTS_WITH GENERAL_ATLANTIC","reference_id":90},{"source_type":"relationship","id":"ONEBRIEF->ODYSSEY_2025","score":0.0,"snippet":"ONEBRIEF INVESTMENT ODYSSEY_2025","reference_id":91},{"source_type":"relationship","id":"ODYSSEY->INSIGHT_PARTNERS","score":0.0,"snippet":"ODYSSEY INVESTS_WITH INSIGHT_PARTNERS","reference_id":92},{"source_type":"relationship","id":"ODYSSEY->APAX_PARTNERS","score":0.0,"snippet":"ODYSSEY INVESTS_WITH APAX_PARTNERS","reference_id":93},{"source_type":"relationship","id":"ONEBRIEF->INSIGHT_PARTNERS","score":0.0,"snippet":"ONEBRIEF INVESTOR INSIGHT_PARTNERS","reference_id":94},{"source_type":"relationship","id":"ODYSSEY->NORDIC_CAPITAL","score":0.0,"snippet":"ODYSSEY INVESTS_WITH NORDIC_CAPITAL","reference_id":95},{"source_type":"relationship","id":"ODYSSEY->SEVEN2","score":0.0,"snippet":"ODYSSEY INVESTS_WITH SEVEN2","reference_id":96},{"source_type":"relationship","id":"AMCS->ODYSSEY_2021","score":0.0,"snippet":"AMCS INVESTS_IN ODYSSEY_2021","reference_id":97},{"source_type":"relationship","id":"QLIK->ODYSSEY_2024","score":0.0,"snippet":"QLIK INVESTS_IN ODYSSEY_2024","reference_id":98},{"source_type":"relationship","id":"ODYSSEY_2022->APAX","score":0.0,"snippet":"ODYSSEY_2022 INVESTS_IN APAX","reference_id":99},{"source_type":"relationship","id":"ODYSSEY_2022->BRIDGEPOINT","score":0.0,"snippet":"ODYSSEY_2022 INVESTS_IN BRIDGEPOINT","reference_id":100},{"source_type":"relationship","id":"APAX_PARTNERS->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"APAX_PARTNERS INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":101},{"source_type":"relationship","id":"HG->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"HG INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":102},{"source_type":"relationship","id":"BRIDGEPOINT->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"BRIDGEPOINT INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":103},{"source_type":"relationship","id":"CRESSEY_&_COMPANY->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"CRESSEY_&_COMPANY INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":104},{"source_type":"relationship","id":"STG_(SOFTWARE_TECHNOLOGY_GROUP)->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"STG_(SOFTWARE_TECHNOLOGY_GROUP) INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":105},{"source_type":"relationship","id":"ACCEL-KKR->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"ACCEL-KKR INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":106},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2022->ACCESS_GROUP","score":0.0,"snippet":"ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH ACCESS_GROUP","reference_id":107},{"source_type":"relationship","id":"ODYSSEY_2021->HG","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN HG","reference_id":108},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2022->AUTHORITY_BRANDS","score":0.0,"snippet":"ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH AUTHORITY_BRANDS","reference_id":109},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2022->IFS","score":0.0,"snippet":"ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH IFS","reference_id":110},{"source_type":"relationship","id":"APAX_PARTNERS->AUTHORITY_BRANDS","score":0.0,"snippet":"APAX_PARTNERS INVESTED_IN AUTHORITY_BRANDS","reference_id":111},{"source_type":"relationship","id":"HG->ACCESS_GROUP","score":0.0,"snippet":"HG INVESTED_IN ACCESS_GROUP","reference_id":112},{"source_type":"relationship","id":"HG->IFS","score":0.0,"snippet":"HG INVESTED_IN IFS","reference_id":113},{"source_type":"relationship","id":"ODYSSEY_2021->APAX","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN APAX","reference_id":114},{"source_type":"relationship","id":"SFDR_ARTICLE_8->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"SFDR_ARTICLE_8 CLASSIFIED_AS ALTAROC_ODYSSEY_2022","reference_id":115},{"source_type":"relationship","id":"AMCS->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"AMCS CO_INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":116},{"source_type":"relationship","id":"NORDIC_CAPITAL->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"NORDIC_CAPITAL CO_INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":117},{"source_type":"relationship","id":"SEVEN2->ALTAROC_ODYSSEY_2022","score":0.0,"snippet":"SEVEN2 CO_INVESTED_IN ALTAROC_ODYSSEY_2022","reference_id":118},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2024->FPCI","score":0.0,"snippet":"ALTAROC_ODYSSEY_2024 USES FPCI","reference_id":119},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2024->THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS","score":0.0,"snippet":"ALTAROC_ODYSSEY_2024 INVESTS_IN THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS","reference_id":120},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2024->QLIK","score":0.0,"snippet":"ALTAROC_ODYSSEY_2024 CO_INVESTS_WITH QLIK","reference_id":121},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2025->FPCI","score":0.0,"snippet":"ALTAROC_ODYSSEY_2025 USES FPCI","reference_id":122},{"source_type":"relationship","id":"ALTAROC->ODYSSEY_2023","score":0.0,"snippet":"ALTAROC OFFERS ODYSSEY_2023","reference_id":123},{"source_type":"relationship","id":"INSIGHT_PARTNERS_XIII->ALTAROC_ODYSSEY_2026","score":0.0,"snippet":"INSIGHT_PARTNERS_XIII MANAGES ALTAROC_ODYSSEY_2026","reference_id":124},{"source_type":"relationship","id":"GBF_XIII->ALTAROC_ODYSSEY_2026","score":0.0,"snippet":"GBF_XIII MANAGES ALTAROC_ODYSSEY_2026","reference_id":125},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2026->INSIGHT_PARTNERS_XIII","score":0.0,"snippet":"ALTAROC_ODYSSEY_2026 INCLUDES INSIGHT_PARTNERS_XIII","reference_id":126},{"source_type":"relationship","id":"ALTAROC_ODYSSEY_2026->INFLEXION","score":0.0,"snippet":"ALTAROC_ODYSSEY_2026 INCLUDES INFLEXION","reference_id":127},{"source_type":"relationship","id":"ODYSSEY_2021->SEVEN2","score":0.0,"snippet":"ODYSSEY_2021 INVESTS_IN SEVEN2","reference_id":128}],"query_mode":"global","retrieval_time_ms":1617}

data: {"type":"done","assistant_message_id":"f938e07e-c095-4b6d-94e5-13e69131f806","tokens_used":1,"duration_ms":1779,"llm_provider":"mistral","llm_model":"mistral-small-latest"}



{
    "conversation": {
        "id": "6216fa00-fa83-4bdf-b996-5d803381de88",
        "tenant_id": "b9d60c6e-c568-487a-a3b9-27318da1549d",
        "workspace_id": "4382e7ab-6fb6-48b6-b49c-c3d1841dcc69",
        "title": "New Conversation",
        "mode": "global",
        "is_pinned": false,
        "is_archived": false,
        "folder_id": null,
        "share_id": null,
        "message_count": null,
        "last_message_preview": null,
        "created_at": "2026-06-29T13:49:05.729373+00:00",
        "updated_at": "2026-06-29T13:49:07.521242+00:00"
    },
    "messages": [
        {
            "id": "60239215-9810-46be-b2c9-51bb6f69168a",
            "conversation_id": "6216fa00-fa83-4bdf-b996-5d803381de88",
            "parent_id": null,
            "role": "user",
            "content": "c'est quoi le produit odyssey 2026 altaroc ?",
            "mode": null,
            "tokens_used": null,
            "duration_ms": null,
            "thinking_time_ms": null,
            "context": null,
            "is_error": false,
            "created_at": "2026-06-29T13:49:05.736480+00:00",
            "updated_at": "2026-06-29T13:49:05.736480+00:00"
        },
        {
            "id": "f938e07e-c095-4b6d-94e5-13e69131f806",
            "conversation_id": "6216fa00-fa83-4bdf-b996-5d803381de88",
            "parent_id": "60239215-9810-46be-b2c9-51bb6f69168a",
            "role": "assistant",
            "content": "",
            "mode": null,
            "tokens_used": 1,
            "duration_ms": 1779,
            "thinking_time_ms": null,
            "context": {
                "sources": [
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6",
                        "title": "altaroc_produits.md",
                        "content": "res** :\n  - Société d'investissement existant depuis plus de 15 ans\n  - Historique de performance 15 ans dans le 1er quartile\n  - Taux de perte historique < 2%\n  - Taille du fonds en levée > 1 Md€ et ",
                        "score": 4.005088806152344,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0",
                        "title": "altaroc_produits.md",
                        "content": "# ALTAROC — CATALOGUE COMPLET DES PRODUITS\n*Dernière mise à jour : Mai 2026 | Source : altaroc.pe*\n\n---\n\n## PRÉSENTATION GÉNÉRALE D'ALTAROC\n\n**Altaroc** (anciennement Amboise Partners, fondée en 1972)",
                        "score": 3.888836622238159,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5",
                        "title": "altaroc_produits.md",
                        "content": ")\n- **Format** : RAIF\n- **Ouverture à la souscription** : 5 mai 2026\n- **Statut** : Ouvert\n- Stratégie alignée sur Odyssey 2026\n\n---\n\n## GAMME 3 — DISCOVERY (Assurance Vie & PER)\n\n### Concept\nLa gamme",
                        "score": 2.0682942867279053,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7",
                        "title": "altaroc_produits.md",
                        "content": " des solutions **sur mesure** :\n- Construction de portefeuilles personnalisés\n- Accès à des gérants spécifiques non disponibles dans les millésimes standards\n- Accompagnement dédié par les équipes d'i",
                        "score": 1.6751633882522583,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4",
                        "title": "altaroc_produits.md",
                        "content": " AUM, 380+ personnes)\n  - **Hg Saturn 4** — LBO Large Cap logiciels, Europe + US (même gérant Hg)\n  - **Insight XIII + GBF XIII** — Growth Equity logiciels à forte croissance (fondé 1995, 80 Mds$ AUM,",
                        "score": 1.3554757833480835,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1",
                        "title": "altaroc_produits.md",
                        "content": " | Perpétuel | Multi-pays |\n\n---\n\n## GAMME 1 — ODYSSEY (Millésimes FPCI Français)\n\n### Concept de base\nLes millésimes Odyssey sont des **Fonds Professionnels de Capital Investissement (FPCI)** non sou",
                        "score": 0.8791240453720093,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3",
                        "title": "altaroc_produits.md",
                        "content": "xpertise logiciels\n  - **New Mountain Capital VII** — Buy-Out nord-américain (tech, santé, services)\n  - **Inflexion Partnership Capital III** — Buy-Out minoritaire, Europe du Nord\n  - **Main Capital ",
                        "score": 0.46388083696365356,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    },
                    {
                        "id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2",
                        "title": "altaroc_produits.md",
                        "content": "ers** — Growth, logiciels en forte croissance\n  - **Apax Partners** — Buy-Out global, multi-secteurs\n  - **Nordic Capital** — Buy-Out pan-européen (santé, tech)\n  - **Seven2** — Buy-Out mid-market eur",
                        "score": 0.17031896114349365,
                        "document_id": "cdfd2960-2444-48e5-8e3b-8b6b1540560a"
                    }
                ],
                "entities": [
                    {
                        "name": "ODYSSEY_2021",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2021, closed with 180M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "INSIGHT_PARTNERS",
                        "entity_type": "ORGANIZATION",
                        "description": "Growth equity investment firm focused on software",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ALTAROC",
                        "entity_type": "ORGANIZATION",
                        "description": "Société française de gestion spécialisée dans le Private Equity pour investisseurs privés avertis.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2026",
                        "entity_type": "EVENT",
                        "description": "6th vintage of Altaroc's Odyssey fund series",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "GENERAL_ATLANTIC",
                        "entity_type": "ORGANIZATION",
                        "description": "Global growth equity investment firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ODYSSEY",
                        "entity_type": "PRODUCT",
                        "description": "Gamme de produits d'Altaroc basée sur des millésimes FPCI annuels.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2022",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2022, closed with 465M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "ALTAROC_DISCOVERY_FCPR",
                        "entity_type": "PRODUCT",
                        "description": "FCPR Evergreen ouvert en permanence accessible via plusieurs supports d'investissement",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2023",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2023, closed with 315M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "ALTALIFE_2023",
                        "entity_type": "PRODUCT",
                        "description": "FCPR millésimé et clôturé accessible via assurance vie française et PER",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"
                        ]
                    },
                    {
                        "name": "ALTAROC_HORIZON_2025",
                        "entity_type": "EVENT",
                        "description": "Closed RAIF fund targeting 50M€",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2025",
                        "entity_type": "EVENT",
                        "description": "5th vintage of ALTAROC ODYSSEY investment fund",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "NORDIC_CAPITAL",
                        "entity_type": "ORGANIZATION",
                        "description": "Pan-European buyout firm focused on healthcare and technology",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "PROGRAMME_RE-UP",
                        "entity_type": "CONCEPT",
                        "description": "Stratégie d'investissement patrimonial long terme d'Altaroc",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-6"
                        ]
                    },
                    {
                        "name": "HG",
                        "entity_type": "ORGANIZATION",
                        "description": "Buy-out firm focused on software in Europe",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2022",
                        "entity_type": "EVENT",
                        "description": "Second vintage of Altaroc Odyssey fund with 465M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ALTAROC_ODYSSEY_2024",
                        "entity_type": "EVENT",
                        "description": "4th vintage of ALTAROC ODYSSEY investment fund",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2024",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2024, closed with 355M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "AMF",
                        "entity_type": "ORGANIZATION",
                        "description": "Autorité des Marchés Financiers, régulateur français des marchés financiers.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "APAX",
                        "entity_type": "ORGANIZATION",
                        "description": "Global buy-out firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF",
                        "entity_type": "PRODUCT",
                        "description": "Fonds secondaire evergreen multi-pays de la gamme Global Evergreen.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "BRIDGEPOINT",
                        "entity_type": "ORGANIZATION",
                        "description": "Pan-European buy-out firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "AMCS",
                        "entity_type": "ORGANIZATION",
                        "description": "Company involved in co-investment (last signed in 2024)",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2026",
                        "entity_type": "EVENT",
                        "description": "Strategic event or initiative referenced in the text",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-5"
                        ]
                    },
                    {
                        "name": "ALTAROC_HORIZON_2026",
                        "entity_type": "EVENT",
                        "description": "Open RAIF fund with subscription opening on 5 May 2026",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "ALTAROC_HORIZON_2024",
                        "entity_type": "EVENT",
                        "description": "Closed ELTIF fund with 8M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "XPERTISE_LOGICIELS",
                        "entity_type": "ORGANIZATION",
                        "description": "Firm managing investment funds",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "DISCOVERY",
                        "entity_type": "PRODUCT",
                        "description": "Gamme de produits d'Altaroc incluant des fonds FCPR et secondaires evergreen.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "APAX_PARTNERS",
                        "entity_type": "ORGANIZATION",
                        "description": "Global buyout investment firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "SEVEN2",
                        "entity_type": "ORGANIZATION",
                        "description": "European mid-market buyout firm",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-1"
                        ]
                    },
                    {
                        "name": "ALTAROC_DISCOVERY",
                        "entity_type": "PRODUCT",
                        "description": "Private Equity access product within French life insurance and PER contracts",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "ACCEL-KKR",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based buy-out firm focused on software growth",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ONEBRIEF",
                        "entity_type": "ORGANIZATION",
                        "description": "Defense software company valued at $1.1B",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "QLIK",
                        "entity_type": "ORGANIZATION",
                        "description": "Business intelligence/data analytics company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "CLEERLY",
                        "entity_type": "ORGANIZATION",
                        "description": "AI cardiovascular company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "AUTHORITY_BRANDS",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based home services franchises company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "SFDR_ARTICLE_8",
                        "entity_type": "LAW_NODE",
                        "description": "Sustainable Finance Disclosure Regulation classification for funds promoting environmental or social characteristics",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "HORIZON_2024",
                        "entity_type": "PRODUCT",
                        "description": "ELTIF investment product from 2024, closed with 8M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "MILLÉSIMES_RAIF_(HORIZON)",
                        "entity_type": "PRODUCT",
                        "description": "Fonds de fonds primaires proposés dans la gamme Odyssey pour investisseurs internationaux.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "ODYSSEY_2025",
                        "entity_type": "PRODUCT",
                        "description": "FPCI investment product from 2025, closed with 300M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "HORIZON_2025",
                        "entity_type": "PRODUCT",
                        "description": "RAIF investment product from 2025, closed with 8M€ raised",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "MILLÉSIMES_FPCI_ANNUELS",
                        "entity_type": "PRODUCT",
                        "description": "Fonds de fonds primaires proposés dans la gamme Odyssey.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "HORIZON_2026",
                        "entity_type": "PRODUCT",
                        "description": "RAIF investment product from 2026, currently open",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-7"
                        ]
                    },
                    {
                        "name": "SPECTER_OPS",
                        "entity_type": "ORGANIZATION",
                        "description": "Cybersecurity company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "STG_(SOFTWARE_TECHNOLOGY_GROUP)",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based buy-out firm focused on software value",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "CRESSEY_&_COMPANY",
                        "entity_type": "ORGANIZATION",
                        "description": "US-based buy-out firm focused on healthcare",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "SCA-SICAV-RAIF",
                        "entity_type": "TECHNOLOGY",
                        "description": "Structure juridique d'investissement (Société en Commandite par Actions, Société d'Investissement à Capital Variable, RAIF).",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "INFLEXION",
                        "entity_type": "ORGANIZATION",
                        "description": "Private equity firm in selection universe",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "FCPR",
                        "entity_type": "TECHNOLOGY",
                        "description": "Fonds Communs de Placement à Risques, structure d'investissement en Private Equity.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "GBF_XIII",
                        "entity_type": "ORGANIZATION",
                        "description": "Growth Equity fund founded in 1995 with $80B AUM",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "RAIF",
                        "entity_type": "TECHNOLOGY",
                        "description": "Reserved Alternative Investment Fund, structure d'investissement luxembourgeoise.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "INSIGHT_PARTNERS_XIII",
                        "entity_type": "ORGANIZATION",
                        "description": "Growth Equity fund focused on high-growth software",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-4"
                        ]
                    },
                    {
                        "name": "TROPHÉE_PATRIMONIA",
                        "entity_type": "EVENT",
                        "description": "Prix récompensant la meilleure relation auprès des CGP en 2024.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "IFS",
                        "entity_type": "ORGANIZATION",
                        "description": "European industrial ERP company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "ACCESS_GROUP",
                        "entity_type": "ORGANIZATION",
                        "description": "UK-based mid-market software company",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    },
                    {
                        "name": "FPCI",
                        "entity_type": "TECHNOLOGY",
                        "description": "Fonds Professionnels de Capital Investissement, structure d'investissement en Private Equity.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "DISCOVERY_FCPR_(EVERGREEN)",
                        "entity_type": "PRODUCT",
                        "description": "Fonds secondaire evergreen de la gamme Discovery accessible via assurance vie/PER/PEA-PME.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS",
                        "entity_type": "ORGANIZATION",
                        "description": "Buy-out fund for large cap/upper mid-market software in North America",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-3"
                        ]
                    },
                    {
                        "name": "ALTALIFE_2023_(FCPR)",
                        "entity_type": "PRODUCT",
                        "description": "Fonds de fonds primaire de la gamme Discovery pour assurance vie/PER en France.",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-0"
                        ]
                    },
                    {
                        "name": "TA_ASSOCIATES_XV",
                        "entity_type": "ORGANIZATION",
                        "description": "Global growth/buy-out firm with strong software expertise",
                        "score": 0.0,
                        "source_chunk_ids": [
                            "cdfd2960-2444-48e5-8e3b-8b6b1540560a-chunk-2"
                        ]
                    }
                ],
                "relationships": [
                    {
                        "source": "ALTAROC",
                        "target": "ODYSSEY",
                        "relation_type": "OFFERS",
                        "description": "ALTAROC OFFERS ODYSSEY",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC",
                        "target": "DISCOVERY",
                        "relation_type": "OFFERS",
                        "description": "ALTAROC OFFERS DISCOVERY",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2022",
                        "target": "HG",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2022 INVESTS_IN HG",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "NORDIC_CAPITAL",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN NORDIC_CAPITAL",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "MILLÉSIMES_FPCI_ANNUELS",
                        "relation_type": "INCLUDES",
                        "description": "ODYSSEY INCLUDES MILLÉSIMES_FPCI_ANNUELS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "MILLÉSIMES_RAIF_(HORIZON)",
                        "relation_type": "INCLUDES",
                        "description": "ODYSSEY INCLUDES MILLÉSIMES_RAIF_(HORIZON)",
                        "score": 0.0
                    },
                    {
                        "source": "DISCOVERY",
                        "target": "ALTALIFE_2023_(FCPR)",
                        "relation_type": "INCLUDES",
                        "description": "DISCOVERY INCLUDES ALTALIFE_2023_(FCPR)",
                        "score": 0.0
                    },
                    {
                        "source": "DISCOVERY",
                        "target": "DISCOVERY_FCPR_(EVERGREEN)",
                        "relation_type": "INCLUDES",
                        "description": "DISCOVERY INCLUDES DISCOVERY_FCPR_(EVERGREEN)",
                        "score": 0.0
                    },
                    {
                        "source": "MILLÉSIMES_FPCI_ANNUELS",
                        "target": "FPCI",
                        "relation_type": "IS_A",
                        "description": "MILLÉSIMES_FPCI_ANNUELS IS_A FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "MILLÉSIMES_RAIF_(HORIZON)",
                        "target": "RAIF",
                        "relation_type": "IS_A",
                        "description": "MILLÉSIMES_RAIF_(HORIZON) IS_A RAIF",
                        "score": 0.0
                    },
                    {
                        "source": "ALTALIFE_2023_(FCPR)",
                        "target": "FCPR",
                        "relation_type": "IS_A",
                        "description": "ALTALIFE_2023_(FCPR) IS_A FCPR",
                        "score": 0.0
                    },
                    {
                        "source": "DISCOVERY_FCPR_(EVERGREEN)",
                        "target": "FCPR",
                        "relation_type": "IS_A",
                        "description": "DISCOVERY_FCPR_(EVERGREEN) IS_A FCPR",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "INSIGHT_PARTNERS",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN INSIGHT_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF",
                        "target": "SCA-SICAV-RAIF",
                        "relation_type": "IS_A",
                        "description": "ALTAROC_GLOBAL_EVERGREEN_SCA-SICAV-RAIF IS_A SCA-SICAV-RAIF",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "GENERAL_ATLANTIC",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN GENERAL_ATLANTIC",
                        "score": 0.0
                    },
                    {
                        "source": "GENERAL_ATLANTIC",
                        "target": "ODYSSEY_2021",
                        "relation_type": "INVESTS_IN",
                        "description": "GENERAL_ATLANTIC INVESTS_IN ODYSSEY_2021",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC",
                        "target": "TROPHÉE_PATRIMONIA",
                        "relation_type": "RECEIVED_AWARD",
                        "description": "ALTAROC RECEIVED_AWARD TROPHÉE_PATRIMONIA",
                        "score": 0.0
                    },
                    {
                        "source": "INSIGHT_PARTNERS",
                        "target": "ODYSSEY_2021",
                        "relation_type": "INVESTS_IN",
                        "description": "INSIGHT_PARTNERS INVESTS_IN ODYSSEY_2021",
                        "score": 0.0
                    },
                    {
                        "source": "MILLÉSIMES_FPCI_ANNUELS",
                        "target": "AMF",
                        "relation_type": "NOT_SUBJECT_TO",
                        "description": "MILLÉSIMES_FPCI_ANNUELS NOT_SUBJECT_TO AMF",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "FPCI",
                        "relation_type": "IS_A",
                        "description": "ODYSSEY IS_A FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "FPCI",
                        "target": "AMF",
                        "relation_type": "NOT_SUBJECT_TO",
                        "description": "FPCI NOT_SUBJECT_TO AMF",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "GENERAL_ATLANTIC",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH GENERAL_ATLANTIC",
                        "score": 0.0
                    },
                    {
                        "source": "ONEBRIEF",
                        "target": "ODYSSEY_2025",
                        "relation_type": "INVESTMENT",
                        "description": "ONEBRIEF INVESTMENT ODYSSEY_2025",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "INSIGHT_PARTNERS",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH INSIGHT_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "APAX_PARTNERS",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH APAX_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ONEBRIEF",
                        "target": "INSIGHT_PARTNERS",
                        "relation_type": "INVESTOR",
                        "description": "ONEBRIEF INVESTOR INSIGHT_PARTNERS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "NORDIC_CAPITAL",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH NORDIC_CAPITAL",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY",
                        "target": "SEVEN2",
                        "relation_type": "INVESTS_WITH",
                        "description": "ODYSSEY INVESTS_WITH SEVEN2",
                        "score": 0.0
                    },
                    {
                        "source": "AMCS",
                        "target": "ODYSSEY_2021",
                        "relation_type": "INVESTS_IN",
                        "description": "AMCS INVESTS_IN ODYSSEY_2021",
                        "score": 0.0
                    },
                    {
                        "source": "QLIK",
                        "target": "ODYSSEY_2024",
                        "relation_type": "INVESTS_IN",
                        "description": "QLIK INVESTS_IN ODYSSEY_2024",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2022",
                        "target": "APAX",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2022 INVESTS_IN APAX",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2022",
                        "target": "BRIDGEPOINT",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2022 INVESTS_IN BRIDGEPOINT",
                        "score": 0.0
                    },
                    {
                        "source": "APAX_PARTNERS",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "APAX_PARTNERS INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "HG",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "HG INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "BRIDGEPOINT",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "BRIDGEPOINT INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "CRESSEY_&_COMPANY",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "CRESSEY_&_COMPANY INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "STG_(SOFTWARE_TECHNOLOGY_GROUP)",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "STG_(SOFTWARE_TECHNOLOGY_GROUP) INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "ACCEL-KKR",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "INVESTED_IN",
                        "description": "ACCEL-KKR INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2022",
                        "target": "ACCESS_GROUP",
                        "relation_type": "CO_INVESTED_WITH",
                        "description": "ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH ACCESS_GROUP",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "HG",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN HG",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2022",
                        "target": "AUTHORITY_BRANDS",
                        "relation_type": "CO_INVESTED_WITH",
                        "description": "ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH AUTHORITY_BRANDS",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2022",
                        "target": "IFS",
                        "relation_type": "CO_INVESTED_WITH",
                        "description": "ALTAROC_ODYSSEY_2022 CO_INVESTED_WITH IFS",
                        "score": 0.0
                    },
                    {
                        "source": "APAX_PARTNERS",
                        "target": "AUTHORITY_BRANDS",
                        "relation_type": "INVESTED_IN",
                        "description": "APAX_PARTNERS INVESTED_IN AUTHORITY_BRANDS",
                        "score": 0.0
                    },
                    {
                        "source": "HG",
                        "target": "ACCESS_GROUP",
                        "relation_type": "INVESTED_IN",
                        "description": "HG INVESTED_IN ACCESS_GROUP",
                        "score": 0.0
                    },
                    {
                        "source": "HG",
                        "target": "IFS",
                        "relation_type": "INVESTED_IN",
                        "description": "HG INVESTED_IN IFS",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "APAX",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN APAX",
                        "score": 0.0
                    },
                    {
                        "source": "SFDR_ARTICLE_8",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CLASSIFIED_AS",
                        "description": "SFDR_ARTICLE_8 CLASSIFIED_AS ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "AMCS",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CO_INVESTED_IN",
                        "description": "AMCS CO_INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "NORDIC_CAPITAL",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CO_INVESTED_IN",
                        "description": "NORDIC_CAPITAL CO_INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "SEVEN2",
                        "target": "ALTAROC_ODYSSEY_2022",
                        "relation_type": "CO_INVESTED_IN",
                        "description": "SEVEN2 CO_INVESTED_IN ALTAROC_ODYSSEY_2022",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2024",
                        "target": "FPCI",
                        "relation_type": "USES",
                        "description": "ALTAROC_ODYSSEY_2024 USES FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2024",
                        "target": "THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS",
                        "relation_type": "INVESTS_IN",
                        "description": "ALTAROC_ODYSSEY_2024 INVESTS_IN THOMA_BRAVO_DISCOVER_V_+_XVI_+_GENESIS",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2024",
                        "target": "QLIK",
                        "relation_type": "CO_INVESTS_WITH",
                        "description": "ALTAROC_ODYSSEY_2024 CO_INVESTS_WITH QLIK",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2025",
                        "target": "FPCI",
                        "relation_type": "USES",
                        "description": "ALTAROC_ODYSSEY_2025 USES FPCI",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC",
                        "target": "ODYSSEY_2023",
                        "relation_type": "OFFERS",
                        "description": "ALTAROC OFFERS ODYSSEY_2023",
                        "score": 0.0
                    },
                    {
                        "source": "INSIGHT_PARTNERS_XIII",
                        "target": "ALTAROC_ODYSSEY_2026",
                        "relation_type": "MANAGES",
                        "description": "INSIGHT_PARTNERS_XIII MANAGES ALTAROC_ODYSSEY_2026",
                        "score": 0.0
                    },
                    {
                        "source": "GBF_XIII",
                        "target": "ALTAROC_ODYSSEY_2026",
                        "relation_type": "MANAGES",
                        "description": "GBF_XIII MANAGES ALTAROC_ODYSSEY_2026",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2026",
                        "target": "INSIGHT_PARTNERS_XIII",
                        "relation_type": "INCLUDES",
                        "description": "ALTAROC_ODYSSEY_2026 INCLUDES INSIGHT_PARTNERS_XIII",
                        "score": 0.0
                    },
                    {
                        "source": "ALTAROC_ODYSSEY_2026",
                        "target": "INFLEXION",
                        "relation_type": "INCLUDES",
                        "description": "ALTAROC_ODYSSEY_2026 INCLUDES INFLEXION",
                        "score": 0.0
                    },
                    {
                        "source": "ODYSSEY_2021",
                        "target": "SEVEN2",
                        "relation_type": "INVESTS_IN",
                        "description": "ODYSSEY_2021 INVESTS_IN SEVEN2",
                        "score": 0.0
                    }
                ]
            },
            "is_error": false,
            "created_at": "2026-06-29T13:49:07.521242+00:00",
            "updated_at": "2026-06-29T13:49:07.528007+00:00"
        }
    ]
}



--------------