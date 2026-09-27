import { Clock3, Lightbulb, Medal } from "lucide-react"

interface GameCompleteStatsProps {
  completionTimeLabel: string
  totalHintsUsed: number
  rankLabel: string
}

export function GameCompleteStats({
  completionTimeLabel,
  totalHintsUsed,
  rankLabel,
}: GameCompleteStatsProps) {
  return (
    <div className="grid grid-cols-3 gap-2 rounded-xl border border-slate-200 bg-slate-50 p-3">
      <div className="rounded-lg bg-white p-2 text-center">
        <div className="mx-auto mb-1 w-fit rounded-full bg-indigo-100 p-1.5 text-indigo-700">
          <Clock3 className="h-3.5 w-3.5" />
        </div>
        <p className="text-[10px] uppercase tracking-wide text-slate-500">Time</p>
        <p className="text-xs font-semibold text-slate-800">{completionTimeLabel}</p>
      </div>
      <div className="rounded-lg bg-white p-2 text-center">
        <div className="mx-auto mb-1 w-fit rounded-full bg-amber-100 p-1.5 text-amber-700">
          <Lightbulb className="h-3.5 w-3.5" />
        </div>
        <p className="text-[10px] uppercase tracking-wide text-slate-500">Hints</p>
        <p className="text-xs font-semibold text-slate-800">{totalHintsUsed}</p>
      </div>
      <div className="rounded-lg bg-white p-2 text-center">
        <div className="mx-auto mb-1 w-fit rounded-full bg-emerald-100 p-1.5 text-emerald-700">
          <Medal className="h-3.5 w-3.5" />
        </div>
        <p className="text-[10px] uppercase tracking-wide text-slate-500">Rank</p>
        <p className="text-xs font-semibold text-slate-800">{rankLabel}</p>
      </div>
    </div>
  )
}
