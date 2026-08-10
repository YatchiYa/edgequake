/**
 * @module entity-type-catalog
 * @description Bilingual (N-language) entity type token catalog for SPEC-096 LAW-L6.
 *
 * Canonical English keys map to localized UPPERCASE_UNDERSCORED tokens so workspace
 * presets stay aligned with Extraction Language. Custom/mixed lists are never remapped.
 *
 * @implements SPEC-096 LAW-L6
 */

import {
  EXTRACTION_LANGUAGES,
  type ExtractionLanguage,
} from '@/constants/extraction-languages';
import {
  ENTITY_PRESETS,
  type PresetKey,
  normalizeEntityType,
} from '@/constants/entity-presets';

/** Languages with localized type tokens (aligned with EXTRACTION_LANGUAGES). */
export type CatalogLanguage = ExtractionLanguage;

type LocalizedTokens = Partial<Record<CatalogLanguage, string>> & {
  English: string;
};

/**
 * Canonical English token → per-language localized token.
 * Missing language entries fall back to English.
 */
export const ENTITY_TYPE_CATALOG: Record<string, LocalizedTokens> = {
  PERSON: {
    English: 'PERSON',
    French: 'PERSONNE',
    Chinese: '人物',
    Japanese: '人物',
    Korean: '인물',
    Spanish: 'PERSONA',
    German: 'PERSON',
    Portuguese: 'PESSOA',
    Italian: 'PERSONA',
    Russian: 'ЛИЧНОСТЬ',
  },
  ORGANIZATION: {
    English: 'ORGANIZATION',
    French: 'ORGANISATION',
    Chinese: '组织',
    Japanese: '組織',
    Korean: '조직',
    Spanish: 'ORGANIZACION',
    German: 'ORGANISATION',
    Portuguese: 'ORGANIZACAO',
    Italian: 'ORGANIZZAZIONE',
    Russian: 'ОРГАНИЗАЦИЯ',
  },
  LOCATION: {
    English: 'LOCATION',
    French: 'LIEU',
    Chinese: '地点',
    Japanese: '場所',
    Korean: '장소',
    Spanish: 'UBICACION',
    German: 'ORT',
    Portuguese: 'LOCAL',
    Italian: 'LUOGO',
    Russian: 'МЕСТО',
  },
  EVENT: {
    English: 'EVENT',
    French: 'EVENEMENT',
    Chinese: '事件',
    Japanese: 'イベント',
    Korean: '이벤트',
    Spanish: 'EVENTO',
    German: 'EREIGNIS',
    Portuguese: 'EVENTO',
    Italian: 'EVENTO',
    Russian: 'СОБЫТИЕ',
  },
  CONCEPT: {
    English: 'CONCEPT',
    French: 'CONCEPT',
    Chinese: '概念',
    Japanese: '概念',
    Korean: '개념',
    Spanish: 'CONCEPTO',
    German: 'KONZEPT',
    Portuguese: 'CONCEITO',
    Italian: 'CONCETTO',
    Russian: 'КОНЦЕПЦИЯ',
  },
  // SPEC-114: Rust default_entity_types() tokens (General preset parity)
  CREATURE: {
    English: 'CREATURE',
    French: 'CREATURE',
    Chinese: '生物',
    Japanese: '生物',
    Korean: '생물',
    Spanish: 'CRIATURA',
    German: 'GESCHOEPF',
    Portuguese: 'CRIATURA',
    Italian: 'CREATURA',
    Russian: 'СУЩЕСТВО',
  },
  CONTENT: {
    English: 'CONTENT',
    French: 'CONTENU',
    Chinese: '内容',
    Japanese: 'コンテンツ',
    Korean: '콘텐츠',
    Spanish: 'CONTENIDO',
    German: 'INHALT',
    Portuguese: 'CONTEUDO',
    Italian: 'CONTENUTO',
    Russian: 'СОДЕРЖАНИЕ',
  },
  DATA: {
    English: 'DATA',
    French: 'DONNEES',
    Chinese: '数据',
    Japanese: 'データ',
    Korean: '데이터',
    Spanish: 'DATOS',
    German: 'DATEN',
    Portuguese: 'DADOS',
    Italian: 'DATI',
    Russian: 'ДАННЫЕ',
  },
  ARTIFACT: {
    English: 'ARTIFACT',
    French: 'ARTEFACT',
    Chinese: '制品',
    Japanese: 'アーティファクト',
    Korean: '아티팩트',
    Spanish: 'ARTEFACTO',
    German: 'ARTEFAKT',
    Portuguese: 'ARTEFATO',
    Italian: 'ARTEFATTO',
    Russian: 'АРТЕФАКТ',
  },
  NATURALOBJECT: {
    English: 'NATURALOBJECT',
    French: 'OBJET_NATUREL',
    Chinese: '自然物',
    Japanese: '自然物',
    Korean: '자연물',
    Spanish: 'OBJETO_NATURAL',
    German: 'NATUROBJEKT',
    Portuguese: 'OBJETO_NATURAL',
    Italian: 'OGGETTO_NATURALE',
    Russian: 'ПРИРОДНЫЙ_ОБЪЕКТ',
  },
  OTHER: {
    English: 'OTHER',
    French: 'AUTRE',
    Chinese: '其他',
    Japanese: 'その他',
    Korean: '기타',
    Spanish: 'OTRO',
    German: 'SONSTIGES',
    Portuguese: 'OUTRO',
    Italian: 'ALTRO',
    Russian: 'ПРОЧЕЕ',
  },
  TECHNOLOGY: {
    English: 'TECHNOLOGY',
    French: 'TECHNOLOGIE',
    Chinese: '技术',
    Japanese: '技術',
    Korean: '기술',
    Spanish: 'TECNOLOGIA',
    German: 'TECHNOLOGIE',
    Portuguese: 'TECNOLOGIA',
    Italian: 'TECNOLOGIA',
    Russian: 'ТЕХНОЛОГИЯ',
  },
  PRODUCT: {
    English: 'PRODUCT',
    French: 'PRODUIT',
    Chinese: '产品',
    Japanese: '製品',
    Korean: '제품',
    Spanish: 'PRODUCTO',
    German: 'PRODUKT',
    Portuguese: 'PRODUTO',
    Italian: 'PRODOTTO',
    Russian: 'ПРОДУКТ',
  },
  DATE: {
    English: 'DATE',
    French: 'DATE',
    Chinese: '日期',
    Japanese: '日付',
    Korean: '날짜',
    Spanish: 'FECHA',
    German: 'DATUM',
    Portuguese: 'DATA',
    Italian: 'DATA',
    Russian: 'ДАТА',
  },
  DOCUMENT: {
    English: 'DOCUMENT',
    French: 'DOCUMENT',
    Chinese: '文档',
    Japanese: '文書',
    Korean: '문서',
    Spanish: 'DOCUMENTO',
    German: 'DOKUMENT',
    Portuguese: 'DOCUMENTO',
    Italian: 'DOCUMENTO',
    Russian: 'ДОКУМЕНТ',
  },
  MACHINE: {
    English: 'MACHINE',
    French: 'MACHINE',
    Chinese: '机器',
    Japanese: '機械',
    Korean: '기계',
    Spanish: 'MAQUINA',
    German: 'MASCHINE',
    Portuguese: 'MAQUINA',
    Italian: 'MACCHINA',
    Russian: 'МАШИНА',
  },
  COMPONENT: {
    English: 'COMPONENT',
    French: 'COMPOSANT',
    Chinese: '组件',
    Japanese: '部品',
    Korean: '부품',
    Spanish: 'COMPONENTE',
    German: 'KOMPONENTE',
    Portuguese: 'COMPONENTE',
    Italian: 'COMPONENTE',
    Russian: 'КОМПОНЕНТ',
  },
  DEFECT: {
    English: 'DEFECT',
    French: 'DEFAUT',
    Chinese: '缺陷',
    Japanese: '欠陥',
    Korean: '결함',
    Spanish: 'DEFECTO',
    German: 'DEFEKT',
    Portuguese: 'DEFEITO',
    Italian: 'DIFETTO',
    Russian: 'ДЕФЕКТ',
  },
  MEASUREMENT: {
    English: 'MEASUREMENT',
    French: 'MESURE',
    Chinese: '测量',
    Japanese: '測定',
    Korean: '측정',
    Spanish: 'MEDICION',
    German: 'MESSUNG',
    Portuguese: 'MEDICAO',
    Italian: 'MISURAZIONE',
    Russian: 'ИЗМЕРЕНИЕ',
  },
  PROCESS: {
    English: 'PROCESS',
    French: 'PROCESSUS',
    Chinese: '流程',
    Japanese: '工程',
    Korean: '공정',
    Spanish: 'PROCESO',
    German: 'PROZESS',
    Portuguese: 'PROCESSO',
    Italian: 'PROCESSO',
    Russian: 'ПРОЦЕСС',
  },
  MATERIAL: {
    English: 'MATERIAL',
    French: 'MATERIAU',
    Chinese: '材料',
    Japanese: '材料',
    Korean: '재료',
    Spanish: 'MATERIAL',
    German: 'MATERIAL',
    Portuguese: 'MATERIAL',
    Italian: 'MATERIALE',
    Russian: 'МАТЕРИАЛ',
  },
  SYMPTOM: {
    English: 'SYMPTOM',
    French: 'SYMPTOME',
    Chinese: '症状',
    Japanese: '症状',
    Korean: '증상',
    Spanish: 'SINTOMA',
    German: 'SYMPTOM',
    Portuguese: 'SINTOMA',
    Italian: 'SINTOMO',
    Russian: 'СИМПТОМ',
  },
  DRUG: {
    English: 'DRUG',
    French: 'MEDICAMENT',
    Chinese: '药物',
    Japanese: '薬剤',
    Korean: '약물',
    Spanish: 'MEDICAMENTO',
    German: 'MEDIKAMENT',
    Portuguese: 'MEDICAMENTO',
    Italian: 'FARMACO',
    Russian: 'ПРЕПАРАТ',
  },
  DIAGNOSIS: {
    English: 'DIAGNOSIS',
    French: 'DIAGNOSTIC',
    Chinese: '诊断',
    Japanese: '診断',
    Korean: '진단',
    Spanish: 'DIAGNOSTICO',
    German: 'DIAGNOSE',
    Portuguese: 'DIAGNOSTICO',
    Italian: 'DIAGNOSI',
    Russian: 'ДИАГНОЗ',
  },
  PROCEDURE: {
    English: 'PROCEDURE',
    French: 'PROCEDURE',
    Chinese: '手术',
    Japanese: '処置',
    Korean: '시술',
    Spanish: 'PROCEDIMIENTO',
    German: 'VERFAHREN',
    Portuguese: 'PROCEDIMENTO',
    Italian: 'PROCEDURA',
    Russian: 'ПРОЦЕДУРА',
  },
  PATIENT: {
    English: 'PATIENT',
    French: 'PATIENT',
    Chinese: '患者',
    Japanese: '患者',
    Korean: '환자',
    Spanish: 'PACIENTE',
    German: 'PATIENT',
    Portuguese: 'PACIENTE',
    Italian: 'PAZIENTE',
    Russian: 'ПАЦИЕНТ',
  },
  CONDITION: {
    English: 'CONDITION',
    French: 'CONDITION',
    Chinese: '病症',
    Japanese: '病態',
    Korean: '상태',
    Spanish: 'CONDICION',
    German: 'ZUSTAND',
    Portuguese: 'CONDICAO',
    Italian: 'CONDIZIONE',
    Russian: 'СОСТОЯНИЕ',
  },
  CONTRACT: {
    English: 'CONTRACT',
    French: 'CONTRAT',
    Chinese: '合同',
    Japanese: '契約',
    Korean: '계약',
    Spanish: 'CONTRATO',
    German: 'VERTRAG',
    Portuguese: 'CONTRATO',
    Italian: 'CONTRATTO',
    Russian: 'ДОГОВОР',
  },
  CLAUSE: {
    English: 'CLAUSE',
    French: 'CLAUSE',
    Chinese: '条款',
    Japanese: '条項',
    Korean: '조항',
    Spanish: 'CLAUSULA',
    German: 'KLAUSEL',
    Portuguese: 'CLAUSULA',
    Italian: 'CLAUSOLA',
    Russian: 'ПУНКТ',
  },
  PARTY: {
    English: 'PARTY',
    French: 'PARTIE',
    Chinese: '当事方',
    Japanese: '当事者',
    Korean: '당사자',
    Spanish: 'PARTE',
    German: 'PARTEI',
    Portuguese: 'PARTE',
    Italian: 'PARTE',
    Russian: 'СТОРОНА',
  },
  REGULATION: {
    English: 'REGULATION',
    French: 'REGLEMENTATION',
    Chinese: '法规',
    Japanese: '規制',
    Korean: '규제',
    Spanish: 'REGULACION',
    German: 'VORSCHRIFT',
    Portuguese: 'REGULAMENTACAO',
    Italian: 'REGOLAMENTO',
    Russian: 'РЕГУЛИРОВАНИЕ',
  },
  JURISDICTION: {
    English: 'JURISDICTION',
    French: 'JURIDICTION',
    Chinese: '管辖',
    Japanese: '管轄',
    Korean: '관할',
    Spanish: 'JURISDICCION',
    German: 'GERICHTSBARKEIT',
    Portuguese: 'JURISDICAO',
    Italian: 'GIURISDIZIONE',
    Russian: 'ЮРИСДИКЦИЯ',
  },
  CASE: {
    English: 'CASE',
    French: 'AFFAIRE',
    Chinese: '案件',
    Japanese: '案件',
    Korean: '사건',
    Spanish: 'CASO',
    German: 'FALL',
    Portuguese: 'CASO',
    Italian: 'CASO',
    Russian: 'ДЕЛО',
  },
  PAPER: {
    English: 'PAPER',
    French: 'ARTICLE',
    Chinese: '论文',
    Japanese: '論文',
    Korean: '논문',
    Spanish: 'ARTICULO',
    German: 'PAPER',
    Portuguese: 'ARTIGO',
    Italian: 'ARTICOLO',
    Russian: 'СТАТЬЯ',
  },
  METHOD: {
    English: 'METHOD',
    French: 'METHODE',
    Chinese: '方法',
    Japanese: '手法',
    Korean: '방법',
    Spanish: 'METODO',
    German: 'METHODE',
    Portuguese: 'METODO',
    Italian: 'METODO',
    Russian: 'МЕТОД',
  },
  DATASET: {
    English: 'DATASET',
    French: 'JEU_DE_DONNEES',
    Chinese: '数据集',
    Japanese: 'データセット',
    Korean: '데이터셋',
    Spanish: 'CONJUNTO_DE_DATOS',
    German: 'DATENSATZ',
    Portuguese: 'CONJUNTO_DE_DADOS',
    Italian: 'DATASET',
    Russian: 'НАБОР_ДАННЫХ',
  },
  HYPOTHESIS: {
    English: 'HYPOTHESIS',
    French: 'HYPOTHESE',
    Chinese: '假设',
    Japanese: '仮説',
    Korean: '가설',
    Spanish: 'HIPOTESIS',
    German: 'HYPOTHESE',
    Portuguese: 'HIPOTESE',
    Italian: 'IPOTESI',
    Russian: 'ГИПОТЕЗА',
  },
  FINDING: {
    English: 'FINDING',
    French: 'RESULTAT',
    Chinese: '发现',
    Japanese: '知見',
    Korean: '발견',
    Spanish: 'HALLAZGO',
    German: 'ERGEBNIS',
    Portuguese: 'ACHADO',
    Italian: 'RISULTATO',
    Russian: 'НАХОДКА',
  },
  METRIC: {
    English: 'METRIC',
    French: 'METRIQUE',
    Chinese: '指标',
    Japanese: '指標',
    Korean: '지표',
    Spanish: 'METRICA',
    German: 'METRIK',
    Portuguese: 'METRICA',
    Italian: 'METRICA',
    Russian: 'МЕТРИКА',
  },
  FUND: {
    English: 'FUND',
    French: 'FONDS',
    Chinese: '基金',
    Japanese: 'ファンド',
    Korean: '펀드',
    Spanish: 'FONDO',
    German: 'FONDS',
    Portuguese: 'FUNDO',
    Italian: 'FONDO',
    Russian: 'ФОНД',
  },
  SECURITY: {
    English: 'SECURITY',
    French: 'TITRE',
    Chinese: '证券',
    Japanese: '証券',
    Korean: '증권',
    Spanish: 'VALOR',
    German: 'WERTPAPIER',
    Portuguese: 'TITULO',
    Italian: 'TITOLO',
    Russian: 'ЦЕННАЯ_БУМАГА',
  },
  RISK: {
    English: 'RISK',
    French: 'RISQUE',
    Chinese: '风险',
    Japanese: 'リスク',
    Korean: '위험',
    Spanish: 'RIESGO',
    German: 'RISIKO',
    Portuguese: 'RISCO',
    Italian: 'RISCHIO',
    Russian: 'РИСК',
  },
  COUNTERPARTY: {
    English: 'COUNTERPARTY',
    French: 'CONTREPARTIE',
    Chinese: '交易对手',
    Japanese: '取引先',
    Korean: '거래상대방',
    Spanish: 'CONTRAPARTE',
    German: 'GEGENPARTEI',
    Portuguese: 'CONTRAPARTE',
    Italian: 'CONTROPARTE',
    Russian: 'КОНТРАГЕНТ',
  },
};

/**
 * Reverse index: any localized token → canonical English key.
 * English keys always win (SPEC-114): e.g. Portuguese DATE→DATA must not
 * steal the English DATA entity type.
 */
const TOKEN_TO_CANONICAL: Map<string, string> = (() => {
  const map = new Map<string, string>();
  for (const canonical of Object.keys(ENTITY_TYPE_CATALOG)) {
    map.set(normalizeEntityType(canonical), canonical);
  }
  for (const [canonical, locales] of Object.entries(ENTITY_TYPE_CATALOG)) {
    for (const token of Object.values(locales)) {
      if (!token) continue;
      const normalized = normalizeEntityType(token);
      if (map.has(normalized)) continue;
      map.set(normalized, canonical);
    }
  }
  return map;
})();

/** Normalize UI/API language (null/undefined/server default) → catalog language. */
export function resolveCatalogLanguage(
  language: string | null | undefined,
): CatalogLanguage {
  if (!language || language === '__server_default__' || language === 'none') {
    return 'English';
  }
  const lower = language.toLowerCase();
  const match = EXTRACTION_LANGUAGES.find((k) => k.toLowerCase() === lower);
  return match ?? 'English';
}

/**
 * Entity types for a domain preset in the given extraction language.
 * `ENTITY_PRESETS[key].types` remain canonical English (backward compatible).
 */
export function getPresetTypes(
  key: Exclude<PresetKey, 'custom'>,
  language?: string | null,
): string[] {
  return localizeTypes(ENTITY_PRESETS[key].types, language);
}

/** Localize one token; unknown tokens pass through normalized. */
export function localizeType(
  token: string,
  language: string | null | undefined,
): string {
  const lang = resolveCatalogLanguage(language);
  const normalized = normalizeEntityType(token);
  const canonical = TOKEN_TO_CANONICAL.get(normalized) ?? normalized;
  const entry = ENTITY_TYPE_CATALOG[canonical];
  if (!entry) return normalized;
  return entry[lang] ?? entry.English;
}

/** Localize a list of tokens (order preserved). */
export function localizeTypes(
  types: string[],
  language: string | null | undefined,
): string[] {
  return types.map((t) => localizeType(t, language));
}

/**
 * Detect which preset matches `types` in **any** catalog language variant.
 * Returns `'custom'` when no preset matches exactly.
 */
export function detectCanonicalPreset(types: string[]): PresetKey {
  const sorted = [...types.map(normalizeEntityType)].filter(Boolean).sort().join(',');
  for (const [key, preset] of Object.entries(ENTITY_PRESETS)) {
    for (const lang of EXTRACTION_LANGUAGES) {
      const localized = localizeTypes(preset.types, lang)
        .map(normalizeEntityType)
        .sort()
        .join(',');
      if (localized === sorted) {
        return key as PresetKey;
      }
    }
  }
  return 'custom';
}

/**
 * Remap a preset-backed type list from one language to another.
 * Returns `null` when the list is custom/mixed (do not rewrite).
 */
export function remapPresetTypes(
  types: string[],
  _fromLang: string | null | undefined,
  toLang: string | null | undefined,
): string[] | null {
  const preset = detectCanonicalPreset(types);
  if (preset === 'custom') return null;
  return localizeTypes(ENTITY_PRESETS[preset].types, toLang);
}
