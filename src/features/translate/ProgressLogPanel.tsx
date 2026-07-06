import type { CSSProperties, RefObject } from "react";
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
      {logs.map((line) => (
        <LogRow key={line.id} line={line} />
      ))}
    </div>
  );
}

function LogRow({ line }: { line: LogLine }) {
  const parsed = parseLogText(line.text);
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

function parseLogText(text: string) {
  const normalized = text.replace(/\s+/g, " ").trim();
  const percent = parsePercent(normalized);
  const counts = [...normalized.matchAll(/(\d+)\/(\d+|--)/g)];
  const usableCount = [...counts]
    .reverse()
    .find((m) => m[2] !== "--" && !isInsideParentheses(normalized, m.index ?? 0));
  const stage = extractStage(normalized);
  const countDetail = usableCount ? `${usableCount[1]}/${usableCount[2]}` : "";
  return {
    text: normalized,
    stage,
    detail: countDetail,
    percent: percent ?? percentFromCount(usableCount),
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
  const beforeCount = text.match(/^(.+?)\s+\d+\/(?:\d+|--)\b/);
  return beforeCount ? beforeCount[1].replace(/[━─—-]+$/g, "").trim() : "";
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
  padding: "10px 12px",
  border: "1px solid rgba(5,5,5,0.08)",
  borderRadius: 6,
  background: "rgba(5,5,5,0.02)",
};

const factValueStyle: CSSProperties = {
  display: "block",
  marginTop: 4,
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
};

const logBoxStyle: CSSProperties = {
  overflow: "auto",
  background: "rgba(5,5,5,0.03)",
  padding: 8,
  borderRadius: 6,
};

const logRowStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "10px minmax(0, 1fr) auto",
  alignItems: "center",
  gap: 8,
  minHeight: 30,
  padding: "4px 6px",
  borderBottom: "1px solid rgba(5,5,5,0.06)",
};

const logDotStyle: CSSProperties = {
  width: 6,
  height: 6,
  borderRadius: "50%",
};

const logTextStyle: CSSProperties = {
  display: "inline-block",
  maxWidth: "100%",
  overflow: "hidden",
  textOverflow: "ellipsis",
  verticalAlign: "bottom",
  whiteSpace: "nowrap",
};

const detailStyle: CSSProperties = {
  marginLeft: 8,
  fontSize: 12,
  whiteSpace: "nowrap",
};
