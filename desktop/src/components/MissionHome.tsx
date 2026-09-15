import { ArrowUpRight } from "lucide-react";
import { NeuralCore } from "./NeuralCore";
import { Feed } from "./Feed";
import { GlassChip, GlassHUD } from "../glass/GlassHUD";
import { newestFirst } from "../chrono";
import type { Activity, Dashboard, Employee, Handoff, SystemEvent, WorkItem } from "../types";

export const eventLabel = (value: string) => value.replaceAll(".", " ").replaceAll("_", " ").replace(/^\w/, (c) => c.toUpperCase());
const time = (value: string) => new Date(value).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
const tone = (event: SystemEvent) => /failed|blocked/.test(event.eventType) ? "warning" : /approval/.test(event.eventType) ? "approval" : /completed/.test(event.eventType) ? "complete" : "neutral";
const count = (n: number) => n.toString().padStart(2, "0");

type Props = {
  dashboard: Dashboard;
  team: Employee[];
  activity: Activity;
  handoffs: Handoff[];
  workItems?: WorkItem[];
  selectEmployee: (id: string) => void;
  selectWork: (id: string) => void;
  openActivity: () => void;
  openApprovals: () => void;
};

export function MissionHome({ dashboard, team, activity, handoffs, workItems, selectEmployee, selectWork, openActivity, openApprovals }: Props) {
  const latest = dashboard.workItems[0];
  const signals = newestFirst(activity.events.filter((e) => e.eventType !== "agent.started"), (e) => e.createdAt).slice(0, 4);
  return (
    <section className="mission-home organism-home">
      <NeuralCore
        employees={team}
        handoffs={handoffs}
        events={activity.events}
        executions={activity.executions}
        workItems={workItems || dashboard.workItems}
        waitingApprovals={dashboard.waitingApprovals}
        onSelect={selectEmployee}
      />
      <GlassHUD className="hud-company" label="Company" title={dashboard.companyState.toLowerCase()}>
        <div className="hud-metrics">
          <div><strong>{count(team.length)}</strong><span>Employees</span></div>
          <div><strong>{count(dashboard.activeAgents.length)}</strong><span>Working</span></div>
        </div>
      </GlassHUD>
      <GlassHUD className="hud-attention" label="Needs Sam" title={`${count(dashboard.waitingApprovals)} waiting`} onClick={openApprovals}>
        <div className="hud-row">
          <GlassChip tone={dashboard.blockedWork ? "warning" : ""}>{count(dashboard.blockedWork)} blocked</GlassChip>
          <GlassChip>{dashboard.providerStatus.ollamaModelAvailable ? dashboard.providerStatus.ollamaModel : "Local model off"}</GlassChip>
        </div>
      </GlassHUD>
      <GlassHUD className="hud-cycle" label="Latest cycle">
        {latest ? (
          <button className="hud-cycle-btn" onClick={() => selectWork(latest.workItemId)}>
            <span className={`cycle-status ${latest.state.toLowerCase()}`}>{latest.state.replaceAll("_", " ")}</span>
            <strong>{latest.title}</strong>
            <small>Owner · {latest.currentOwner} <ArrowUpRight size={14} /></small>
          </button>
        ) : <p className="quiet-note">Ready for your first signal.</p>}
      </GlassHUD>
      <GlassHUD className="hud-activity" label="Live signals">
        <Feed className="hud-feed" pinKey={`hud:${signals[0]?.eventId || ""}:${signals.length}`}>
          {signals.length ? signals.map((e) => (
            <button className="signal-item" key={e.eventId} onClick={() => e.workItemId ? selectWork(e.workItemId) : openActivity()}>
              <div><span className={`signal-dot ${tone(e)}`} /><span className="actor">{e.actor}</span><time>{time(e.createdAt)}</time></div>
              <strong>{eventLabel(e.eventType)}</strong>
            </button>
          )) : <p className="quiet-note">The organism is quiet.</p>}
        </Feed>
        <button className="text-link" onClick={openActivity}>Full activity</button>
      </GlassHUD>
    </section>
  );
}

export function EmployeeTeam({ team, activity, onNovaResearch }: { team: Employee[]; activity: Activity; onNovaResearch?: () => void }) {
  const events = newestFirst(activity.events, (e) => e.createdAt);
  const runs = newestFirst(activity.runs, (r) => r.startedAt);
  return (
    <>
      <div className="page-intro">
        <span className="eyebrow">Workforce</span>
        <h2>Specialists. Working as one.</h2>
        <p>Registry-backed employees. New hires appear here and in the Neural Core without a frontend edit.</p>
      </div>
      <section className="team-grid">
        {team.map((e, index) => {
          const run = runs.find((r) => r.agent === e.employeeId && !r.finishedAt);
          const event = events.find((a) => a.actor === e.employeeId);
          return (
            <article className="employee-card glass-hud" key={e.employeeId}>
              <div className="employee-top">
                <div className={`employee-orb ${e.runtimeStatus.toLowerCase()}`}><span>{e.name.slice(0, 1)}</span></div>
                <span className="employee-index">{count(index + 1)}</span>
                <span className={`state state-${e.runtimeStatus.toLowerCase()}`}>{e.runtimeStatus}</span>
              </div>
              <h2>{e.name}</h2>
              <p className="employee-role">{e.title}</p>
              <div className="employee-report">Reports to <strong>{e.reportsTo}</strong></div>
              <div className="employee-current">
                <span className="eyebrow">{run ? "WORKING ON" : "LAST SIGNAL"}</span>
                <strong>{run ? eventLabel(run.taskType) : event ? eventLabel(event.eventType) : "Ready for an assignment"}</strong>
                <small>{run ? `${run.provider} · ${run.model}` : event ? time(event.createdAt) : "No recorded activity"}</small>
              </div>
              {e.employeeId === "nova" && onNovaResearch ? <button className="text-link" onClick={onNovaResearch}>Research Instagram creator →</button> : null}
            </article>
          );
        })}
      </section>
    </>
  );
}
