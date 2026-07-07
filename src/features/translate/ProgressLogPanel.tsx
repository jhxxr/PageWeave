import { useMemo, type CSSProperties, type RefObject } from "react";
import { Empty, Progress, Space, Tag, Typography } from "antd";
import type { ProgressProps } from "antd";
import type { LogLine, TaskStatus } from "../../stores/translateStore";

const { Text } = Typography;

export const statusColor: Record<string, string> = {
  idle: "default",
  running: "processing",
  success: "success",
  error: "error",
  cancelled: "warning",
};

export function latestReadableLog(logs: LogLine[]): string | undefined {
  const displayLogs = compactProgressLogs(logs);
  for (let i = displayLogs.length - 1; i >= 0; i -= 1) {
    const parsed = displayLogs[i].parsed;
    if (!parsed.stage) return parsed.text;
    return parsed.stage;
  }
  return undefined;
}

interface ProgressOverviewProps {
  percent: number;
  status: TaskStatus;
  stage: string;
  latestLog?: string;
  stageLabel: string;
  latestLabel: string;
}

export function ProgressOverview({
  percent,
  status,
  stage,
  latestLog,
  stageLabel,
  latestLabel,
}: ProgressOverviewProps) {
  const readableLatest = latestLog ? summarizeLogText(latestLog) : "";
  return (
    <Space direction="vertical" size={12} style={{ width: "100%" }}>
      <Progress
        percent={percent}
        strokeWidth={10}
        status={progressStatus(status)}
      />
      <div style={overviewGridStyle}>
        <ProgressFact label={stageLabel} value={stage || "-"} />
        <ProgressFact label={latestLabel} value={readableLatest || "-"} />
      </div>
    </Space>
  );
}

interface ProgressFactProps {
  label: string;
  value: string;
}

function ProgressFact({ label, value }: ProgressFactProps) {
  return (
    <div style={factStyle}>
      <Text type="secondary" style={{ fontSize: 12 }}>
        {label}
      </Text>
      <Text style={factValueStyle}>{value}</Text>
    </div>
  );
}

interface LogStreamProps {
  logs: LogLine[];
  emptyText: string;
  height?: number;
  containerRef?: RefObject<HTMLDivElement | null>;
}

export function LogStream({ logs, emptyText, height = 240, containerRef }: LogStreamProps) {
  const displayLogs = useMemo(() => compactProgressLogs(logs), [logs]);

  if (logs.length === 0) {
    return (
      <div
        ref={containerRef}
        style={{ ...logBoxStyle, height, display: "grid", placeItems: "center" }}
      >
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={emptyText} />
      </div>
    );
  }

  return (
    <div ref={containerRef} style={{ ...logBoxStyle, height }}>
      {displayLogs.map((line) => (
        <LogRow key={line.id} line={line} />
      ))}
    </div>
  );
}

function LogRow({ line }: { line: DisplayLogLine }) {
  const parsed = line.parsed;
  const tone = line.stream === "stderr" ? "#d46b08" : "#1677ff";

  return (
    <div style={logRowStyle}>
      <span style={{ ...logDotStyle, background: tone }} />
      <div style={{ minWidth: 0 }}>
        <Text strong={!!parsed.stage} style={logTextStyle}>
          {parsed.stage || parsed.text}
        </Text>
        {parsed.detail && (
          <Text type="secondary" style={detailStyle}>
            {parsed.detail}
          </Text>
        )}
      </div>
      {parsed.percent != null && (
        <Tag color={parsed.percent >= 100 ? "success" : "processing"} style={{ marginRight: 0 }}>
          {parsed.percent}%
        </Tag>
      )}
    </div>
  );
}

interface ParsedLogText {
  text: string;
  stage: string;
  detail: string;
  percent: number | undefined;
  current: number | undefined;
  total: number | undefined;
}

interface DisplayLogLine extends LogLine {
  parsed: ParsedLogText;
}

function compactProgressLogs(logs: LogLine[]): DisplayLogLine[] {
  const rows: DisplayLogLine[] = [];
  const progressIndexByStage = new Map<string, number>();

  for (const line of logs) {
    const parsed = parseLogText(line.text);
    if (isProgressFrame(parsed)) continue;
    if (mergeContinuationLog(rows, line, parsed)) continue;
    const key = progressLogKey(parsed);
    if (!key) {
      rows.push({ ...line, parsed });
      continue;
    }

    const existingIndex = progressIndexByStage.get(key);
    if (existingIndex == null) {
      progressIndexByStage.set(key, rows.length);
      rows.push({ ...line, parsed });
      continue;
    }

    const existing = rows[existingIndex];
    if (isCompleteProgress(existing.parsed)) {
      if (parsed.percent != null && parsed.percent < 100) {
        progressIndexByStage.set(key, rows.length);
        rows.push({ ...line, parsed });
      }
      continue;
    }

    rows[existingIndex] = {
      ...line,
      id: existing.id,
      parsed,
    };
  }

  return rows;
}

function mergeContinuationLog(
  rows: DisplayLogLine[],
  line: LogLine,
  parsed: ParsedLogText,
): boolean {
  if (rows.length === 0 || line.stream !== "stderr" || parsed.stage) return false;
  if (!looksLikeContinuation(parsed.text)) return false;
  const previous = rows[rows.length - 1];
  if (previous.stream !== line.stream || previous.parsed.stage) return false;
  const mergedText = `${previous.parsed.text} ${parsed.text}`.replace(/\s+/g, " ").trim();
  if (mergedText.length > 260) return false;
  rows[rows.length - 1] = {
    ...previous,
    text: mergedText,
    parsed: parseLogText(mergedText),
  };
  return true;
}

function looksLikeContinuation(text: string): boolean {
  if (!text || text.length > 90) return false;
  return /^[a-z(,.;:]/.test(text) || /^(line|column|value|extract|automatic|translation|fallback|during)\b/i.test(text);
}

function progressLogKey(parsed: ParsedLogText): string {
  return parsed.stage;
}

function isCompleteProgress(parsed: ParsedLogText): boolean {
  return parsed.percent != null && parsed.percent >= 100;
}

export function cap(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function progressStatus(status: TaskStatus): ProgressProps["status"] {
  if (status === "error") return "exception";
  if (status === "success") return "success";
  return undefined;
}

function summarizeLogText(text: string): string {
  const parsed = parseLogText(text);
  if (!parsed.stage) return parsed.text;
  return parsed.detail ? `${parsed.stage} · ${parsed.detail}` : parsed.stage;
}

function parseLogText(text: string): ParsedLogText {
  const normalized = text.replace(/\s+/g, " ").trim();
  const percent = parsePercent(normalized);
  const richCount = parseRichProgressCount(normalized);
  const counts = [...normalized.matchAll(/(\d+)\/(\d+|--)/g)];
  const usableCount = [...counts]
    .reverse()
    .find((m) => m[2] !== "--" && !isInsideParentheses(normalized, m.index ?? 0));
  const stage = extractStage(normalized);
  const current = richCount?.current ?? countNumber(usableCount?.[1]);
  const total = richCount?.total ?? countNumber(usableCount?.[2]);
  const countDetail = formatCountDetail(current, total);
  return {
    text: normalized,
    stage,
    detail: countDetail,
    percent: percent ?? percentFromCount(usableCount) ?? percentFromRichCount(richCount),
    current,
    total,
  };
}

function parsePercent(text: string): number | undefined {
  const match = text.match(/(\d{1,3})%/);
  if (!match) return undefined;
  const value = Number(match[1]);
  return value <= 100 ? value : undefined;
}

function percentFromCount(match: RegExpMatchArray | undefined): number | undefined {
  if (!match || match[2] === "--") return undefined;
  const current = Number(match[1]);
  const total = Number(match[2]);
  if (!Number.isFinite(current) || !Number.isFinite(total) || total <= 0) {
    return undefined;
  }
  return Math.min(100, Math.floor((Math.min(current, total) / total) * 100));
}

function extractStage(text: string): string {
  const beforeBar = text.match(/^(.+?)\s+\(\d+\/\d+\)\s+/);
  if (beforeBar) return beforeBar[1].trim();
  const beforeRichCount = text.match(/^(.+?)\s+-+\s+\d+(?:[/?](?:\d+|--))?/);
  if (beforeRichCount) return beforeRichCount[1].trim();
  const beforeCount = text.match(/^(.+?)\s+\d+\/(?:\d+|--)\b/);
  return beforeCount ? beforeCount[1].replace(/-+$/g, "").trim() : "";
}

interface RichProgressCount {
  current: number;
  total: number | undefined;
}

function parseRichProgressCount(text: string): RichProgressCount | undefined {
  const match = text.match(/^.+?\s+-+\s+(\d+)(?:[/?](\d+|--))?/);
  if (!match) return undefined;
  const current = Number(match[1]);
  const total = match[2] && match[2] !== "--" ? Number(match[2]) : undefined;
  if (!Number.isFinite(current)) return undefined;
  return { current, total: Number.isFinite(total) ? total : undefined };
}

function percentFromRichCount(count: RichProgressCount | undefined): number | undefined {
  if (!count?.total || count.total <= 0) return undefined;
  return Math.min(100, Math.floor((Math.min(count.current, count.total) / count.total) * 100));
}

function countNumber(value: string | undefined): number | undefined {
  if (!value || value === "--") return undefined;
  const n = Number(value);
  return Number.isFinite(n) ? n : undefined;
}

function formatCountDetail(current: number | undefined, total: number | undefined): string {
  if (current == null) return "";
  return total == null ? `${current}` : `${current}/${total}`;
}

function isProgressFrame(parsed: ParsedLogText): boolean {
  return parsed.stage === "translate" && parsed.percent != null;
}

function isInsideParentheses(text: string, index: number): boolean {
  const open = text.lastIndexOf("(", index);
  const close = text.lastIndexOf(")", index);
  return open > close;
}

const overviewGridStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
  gap: 12,
};

const factStyle: CSSProperties = {
  minWidth: 0,
  padding: "10px 14px",
  border: "1px solid rgba(99, 102, 241, 0.15)",
  borderRadius: 10,
  background: "rgba(99, 102, 241, 0.02)",
};

const factValueStyle: CSSProperties = {
  display: "block",
  marginTop: 4,
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
  fontWeight: 600,
};

const logBoxStyle: CSSProperties = {
  overflow: "auto",
  background: "#0f172a",
  padding: "12px 16px",
  borderRadius: 12,
  border: "1px solid #1e293b",
  boxShadow: "inset 0 4px 12px rgba(0, 0, 0, 0.4)",
  fontFamily: "'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace",
};

const logRowStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "10px minmax(0, 1fr) auto",
  alignItems: "center",
  gap: 8,
  minHeight: 28,
  padding: "6px 8px",
  borderBottom: "1px solid rgba(255, 255, 255, 0.05)",
};

const logDotStyle: CSSProperties = {
  width: 6,
  height: 6,
  borderRadius: "50%",
  boxShadow: "0 0 8px currentColor",
};

const logTextStyle: CSSProperties = {
  display: "inline-block",
  maxWidth: "100%",
  overflow: "hidden",
  textOverflow: "ellipsis",
  verticalAlign: "bottom",
  whiteSpace: "nowrap",
  color: "#f1f5f9",
};

const detailStyle: CSSProperties = {
  marginLeft: 8,
  fontSize: 12,
  whiteSpace: "nowrap",
  color: "#94a3b8",
};
