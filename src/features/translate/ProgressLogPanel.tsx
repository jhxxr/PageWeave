import { type CSSProperties, type RefObject } from "react";
import { Empty, Progress, Space, Typography } from "antd";
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
  showLatest?: boolean;
}

export function ProgressOverview({
  percent,
  status,
  stage,
  latestLog,
  stageLabel,
  latestLabel,
  showLatest = true,
}: ProgressOverviewProps) {
  return (
    <Space direction="vertical" size={12} style={{ width: "100%" }}>
      <Progress
        percent={percent}
        strokeWidth={10}
        status={progressStatus(status)}
      />
      <div style={overviewGridStyle}>
        <ProgressFact label={stageLabel} value={stage || "-"} />
        {showLatest && <ProgressFact label={latestLabel} value={latestLog || "-"} />}
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
  const tone = line.stream === "stderr" ? "#d46b08" : "#1677ff";

  return (
    <div style={logRowStyle}>
      <span style={{ ...logDotStyle, background: tone }} />
      <div style={{ minWidth: 0 }}>
        <Text style={logTextStyle}>{line.text}</Text>
      </div>
    </div>
  );
}

export function cap(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function progressStatus(status: TaskStatus): ProgressProps["status"] {
  if (status === "error") return "exception";
  if (status === "success") return "success";
  // Keep the bar animated while running even at 0% so users see activity.
  if (status === "running") return "active";
  return undefined;
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
  gridTemplateColumns: "10px minmax(0, 1fr)",
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
