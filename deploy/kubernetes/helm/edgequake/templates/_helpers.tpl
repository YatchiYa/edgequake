{{/*
EdgeQuake Helm helpers — DRY labels and image tags (LAW-138-1).
*/}}
{{- define "edgequake.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "edgequake.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{- define "edgequake.namespace" -}}
{{- .Values.namespace.name }}
{{- end }}

{{- define "edgequake.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "edgequake.labels" -}}
helm.sh/chart: {{ include "edgequake.chart" . }}
{{ include "edgequake.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "edgequake.selectorLabels" -}}
app.kubernetes.io/name: {{ include "edgequake.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "edgequake.api.selectorLabels" -}}
{{ include "edgequake.selectorLabels" . }}
app.kubernetes.io/component: api
{{- end }}

{{- define "edgequake.web.selectorLabels" -}}
{{ include "edgequake.selectorLabels" . }}
app.kubernetes.io/component: web
{{- end }}

{{- define "edgequake.postgres.selectorLabels" -}}
{{ include "edgequake.selectorLabels" . }}
app.kubernetes.io/component: postgres
{{- end }}

{{- define "edgequake.version" -}}
{{- .Values.global.edgequakeVersion | default .Chart.AppVersion }}
{{- end }}

{{- define "edgequake.api.image" -}}
{{- $tag := .Values.api.image.tag | default (include "edgequake.version" .) }}
{{- printf "%s:%s" .Values.api.image.repository $tag }}
{{- end }}

{{- define "edgequake.web.image" -}}
{{- $tag := .Values.web.image.tag | default (include "edgequake.version" .) }}
{{- printf "%s:%s" .Values.web.image.repository $tag }}
{{- end }}

{{- define "edgequake.postgres.image" -}}
{{- $tag := .Values.postgres.image.tag | default (include "edgequake.version" .) }}
{{- printf "%s:%s" .Values.postgres.image.repository $tag }}
{{- end }}

{{- define "edgequake.postgres.host" -}}
edgequake-postgres.{{ include "edgequake.namespace" . }}.svc.cluster.local
{{- end }}

{{- define "edgequake.api.serviceName" -}}
edgequake-api
{{- end }}

{{- define "edgequake.web.serviceName" -}}
edgequake-web
{{- end }}

{{- define "edgequake.databaseUrl" -}}
postgres://{{ .Values.postgres.auth.username }}:{{ .Values.postgres.auth.password }}@edgequake-postgres.{{ include "edgequake.namespace" . }}.svc.cluster.local:5432/{{ .Values.postgres.auth.database }}
{{- end }}
