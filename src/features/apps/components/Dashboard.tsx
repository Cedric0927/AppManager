import React, { useState } from "react";
import ReactECharts from "echarts-for-react";

const COLORS = {
  InstallLocation: "#6366f1", // indigo-500
  AppData: "#f59e0b", // amber-500
  ProgramData: "#10b981", // emerald-500
  Unknown: "#71717a", // zinc-500
};

const LABELS: Record<string, string> = {
  InstallLocation: "安装位置",
  AppData: "应用数据 (AppData)",
  ProgramData: "共享数据 (ProgramData)",
  Unknown: "其他",
};

interface DashboardProps {
  stats: {
    categoryData: { name: string; value: number }[];
    topAppsData: any[];
  };
  totalApps: number;
  formatBytes: (bytes: number) => string;
  isScanning: boolean;
}

export function Dashboard({ stats, totalApps, formatBytes, isScanning }: DashboardProps) {
  const { categoryData, topAppsData } = stats;
  const [hoveredCategory, setHoveredCategory] = useState<{
    name: string;
    value: number;
    percent: number;
  } | null>(null);

  const hasData = categoryData.length > 0 || topAppsData.length > 0;
  const totalBytes = categoryData.reduce((acc, item) => acc + item.value, 0);
  const appDataSize = categoryData.find((c) => c.name === "AppData")?.value || 0;

  // 1. 骨架屏配置（仅在完全无数据且未扫描时显示）
  const skeletonOption = {
    backgroundColor: "transparent",
    xAxis: { type: "value", show: false, max: 100 },
    yAxis: {
      type: "category",
      data: ["", "", "", "", ""],
      inverse: true,
      axisLine: { show: false },
      axisTick: { show: false },
    },
    series: [
      {
        type: "bar",
        barWidth: 14,
        silent: true,
        itemStyle: { color: "rgba(255, 255, 255, 0.03)", borderRadius: 4 },
        data: [100, 85, 70, 55, 40],
      },
    ],
  };

  // 2. 真实数据配置
  const pieOption = {
    backgroundColor: "transparent",
    tooltip: {
      show: false, // 关闭原生 tooltip，因为我们有中心显示和下方列表
    },
    series: [
      {
        name: "空间占用分布",
        type: "pie",
        radius: ["65%", "85%"], // 稍微调大一点，给中心留更多空间
        center: ["50%", "50%"],
        avoidLabelOverlap: false,
        itemStyle: {
          borderRadius: 4,
          borderColor: "rgba(9, 9, 11, 1)",
          borderWidth: 2,
        },
        label: { show: false },
        emphasis: {
          scale: true,
          scaleSize: 8,
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: "rgba(0, 0, 0, 0.5)",
          },
        },
        data: categoryData.map((item) => ({
          value: item.value,
          name: item.name,
          itemStyle: {
            color: COLORS[item.name as keyof typeof COLORS] || COLORS.Unknown,
          },
        })),
      },
    ],
  };

  const onChartEvents = {
    mouseover: (params: any) => {
      setHoveredCategory({
        name: params.name,
        value: params.value,
        percent: params.percent,
      });
    },
    mouseout: () => {
      setHoveredCategory(null);
    },
  };

  const barOption = {
    backgroundColor: "transparent",
    animationDuration: 300,
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "line", lineStyle: { color: "rgba(255, 255, 255, 0.1)", width: 1 } },
      backgroundColor: "rgba(9, 9, 11, 0.95)",
      borderColor: "rgba(255, 255, 255, 0.1)",
      borderWidth: 1,
      textStyle: { color: "#e4e4e7", fontSize: 12 },
      formatter: (params: any[]) => {
        if (!params.length) return "";
        const appName = params[0].axisValue;
        const appData = topAppsData.find((d) => d.name === appName);
        if (!appData) return "";

        let html = `<div style="font-weight: 700; margin-bottom: 8px; color: #fff">${appName}</div>`;
        Object.keys(LABELS).forEach((key) => {
          const val = appData[key] || 0;
          if (val > 0) {
            html += `
              <div style="display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 4px">
                <span style="color: ${COLORS[key as keyof typeof COLORS]}; font-size: 11px">● ${LABELS[key]}</span>
                <span style="font-family: monospace; color: #d4d4d8; font-size: 11px">${formatBytes(val)}</span>
              </div>
            `;
          }
        });
        html += `
          <div style="margin-top: 8px; padding-top: 8px; border-top: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: space-between">
            <span style="font-weight: 600; color: #fff">总占用</span>
            <span style="font-weight: 700; color: #fff">${formatBytes(appData.totalBytes)}</span>
          </div>
        `;
        return html;
      },
    },
    grid: { top: 10, left: 10, right: 40, bottom: 10, containLabel: true },
    xAxis: {
      type: "value",
      show: false,
      splitLine: { show: false },
      max: "dataMax", // 显式设置 max，避免继承骨架屏的 100
    },
    yAxis: {
      type: "category",
      data: topAppsData.map((d) => d.name),
      inverse: true,
      axisLine: { show: false },
      axisTick: { show: false },
      axisLabel: { color: "#a1a1aa", fontSize: 11, width: 120, overflow: "truncate" },
    },
    series: [
      {
        name: LABELS.InstallLocation,
        type: "bar",
        stack: "total",
        barWidth: 14,
        itemStyle: { color: COLORS.InstallLocation },
        data: topAppsData.map((d) => d.InstallLocation || 0),
      },
      {
        name: LABELS.AppData,
        type: "bar",
        stack: "total",
        barWidth: 14,
        itemStyle: { color: COLORS.AppData },
        data: topAppsData.map((d) => d.AppData || 0),
      },
      {
        name: LABELS.ProgramData,
        type: "bar",
        stack: "total",
        barWidth: 14,
        itemStyle: { color: COLORS.ProgramData },
        data: topAppsData.map((d) => d.ProgramData || 0),
      },
      {
        name: LABELS.Unknown,
        type: "bar",
        stack: "total",
        barWidth: 14,
        itemStyle: { color: COLORS.Unknown, borderRadius: [0, 4, 4, 0] },
        data: topAppsData.map((d) => d.Unknown || 0),
      },
    ],
  };

  return (
    <div className="flex flex-col gap-6">
      {/* 顶部统计指标 */}
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-1 rounded-2xl bg-zinc-900/40 p-4 ring-1 ring-white/10 backdrop-blur-sm">
          <span className="text-[10px] font-medium text-zinc-500 uppercase tracking-wider">
            已扫描应用
          </span>
          <div className="flex items-baseline gap-1">
            <span className="text-xl font-bold text-white font-mono">{totalApps}</span>
            <span className="text-[10px] text-zinc-500">个</span>
          </div>
        </div>
        <div className="flex flex-col gap-1 rounded-2xl bg-zinc-900/40 p-4 ring-1 ring-white/10 backdrop-blur-sm">
          <span className="text-[10px] font-medium text-zinc-500 uppercase tracking-wider">
            应用数据 (AppData)
          </span>
          <div className="flex items-baseline gap-1">
            <span className="text-xl font-bold text-amber-500 font-mono">
              {formatBytes(appDataSize).split(" ")[0]}
            </span>
            <span className="text-[10px] text-zinc-500">
              {formatBytes(appDataSize).split(" ")[1]}
            </span>
          </div>
        </div>
      </div>

      {/* 存储成分饼图 */}
      <div className="flex flex-col gap-4 rounded-2xl bg-zinc-900/40 p-5 ring-1 ring-white/10 backdrop-blur-sm">
        <div className="flex items-center justify-between">
          <div className="text-sm font-semibold text-zinc-100">空间占用分布</div>
          {isScanning && (
            <div className="flex items-center gap-1.5">
              <div className="h-1.5 w-1.5 animate-pulse rounded-full bg-indigo-500" />
              <span className="text-[10px] font-medium text-zinc-500">实时计算中</span>
            </div>
          )}
        </div>

        <div className="h-[220px] w-full relative">
          {!hasData && !isScanning ? (
            <div className="absolute inset-0 flex items-center justify-center z-10">
              <span className="text-xs text-zinc-600">等待扫描数据...</span>
            </div>
          ) : null}

          {/* 中心自定义内容 */}
          {hasData && (
            <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none z-10">
              <div className="text-center">
                <div className="text-[10px] text-zinc-500 uppercase tracking-wider mb-0.5">
                  {hoveredCategory ? LABELS[hoveredCategory.name] || hoveredCategory.name : "总占用"}
                </div>
                <div className="text-lg font-bold text-white font-mono leading-tight">
                  {hoveredCategory ? formatBytes(hoveredCategory.value) : formatBytes(totalBytes)}
                </div>
                {hoveredCategory && (
                  <div className="text-[10px] text-indigo-400 font-medium mt-0.5">
                    占比 {hoveredCategory.percent}%
                  </div>
                )}
              </div>
            </div>
          )}

          <ReactECharts
            key={hasData ? "pie-real" : "pie-skeleton"}
            option={hasData ? pieOption : skeletonOption}
            onEvents={onChartEvents}
            style={{ height: "100%", width: "100%" }}
            opts={{ renderer: "canvas" }}
            notMerge={true}
          />
        </div>

        {/* 详细列表 */}
        {hasData && (
          <div className="flex flex-col gap-2 pt-2">
            {categoryData.map((item) => (
              <div
                key={item.name}
                className="flex items-center justify-between text-[11px] group transition-all"
              >
                <div className="flex items-center gap-2">
                  <div
                    className="h-1.5 w-1.5 rounded-full"
                    style={{ backgroundColor: COLORS[item.name as keyof typeof COLORS] }}
                  />
                  <span className="text-zinc-400 group-hover:text-zinc-200 transition-colors">
                    {LABELS[item.name] || item.name}
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="font-mono text-zinc-500">
                    {((item.value / totalBytes) * 100).toFixed(1)}%
                  </span>
                  <span className="font-mono text-zinc-300 font-medium">
                    {formatBytes(item.value)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Top 10 应用动态条形图 */}
      <div className="flex flex-col gap-4 rounded-2xl bg-zinc-900/40 p-5 ring-1 ring-white/10 backdrop-blur-sm">
        <div className="flex items-center justify-between">
          <div className="text-sm font-semibold text-zinc-100">占用 Top 10 应用</div>
        </div>
        <div className="h-[400px] w-full relative">
          {!hasData && !isScanning ? (
            <div className="absolute inset-0 flex items-center justify-center z-10">
              <span className="text-xs text-zinc-600">暂无应用排名</span>
            </div>
          ) : null}
          <ReactECharts
            key={hasData ? "bar-real" : "bar-skeleton"}
            option={hasData ? barOption : skeletonOption}
            style={{ height: "100%", width: "100%" }}
            opts={{ renderer: "canvas" }}
            notMerge={true}
          />
        </div>
      </div>
    </div>
  );
}
