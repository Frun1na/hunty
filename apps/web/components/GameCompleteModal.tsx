"use client"

import { useRef } from "react"

import { Button } from "@hunty/ui"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import Coin from "@/components/icons/Coin"
import Replay from "@/components/icons/Replay"
import { RewardsPanel } from "@/components/RewardsPanel"
import { NftMintProgress } from "@/components/NftMintProgress"
import { AchievementCertificate } from "@/components/AchievementCertificate"
import { LevelUpModal } from "./LevelUpModal"
import { useQuery } from "@tanstack/react-query"
import { checkRegistrationStatus } from "@/lib/contracts/player-registration"
import { SOROBAN_READ_STALE_TIME_MS } from "@/lib/soroban/queryConfig"
import { useXlmUsdPrice } from "@/hooks/useXlmUsdPrice"
import type { RewardReceipt } from "@/lib/types"
import { queryCachePolicy, queryKeys } from "@/lib/queryKeys"
import { formatDuration } from "@/lib/huntAttemptHistory"

import { GameCompleteStats } from "./GameCompleteStats"
import { ShareAchievementMenu } from "./ShareAchievementMenu"
import { ShareResultCardMenu } from "./ShareResultCardMenu"
import { HuntReviewForm } from "./HuntReviewForm"
import { useGameCompleteEffects } from "./useGameCompleteEffects"
import { GameCompleteAchievements } from "./GameCompleteAchievements"
import { GameCompleteRewardReceipt } from "./GameCompleteRewardReceipt"

interface GameCompleteModalProps {
  isOpen: boolean
  onClose: () => void
  onGoHome: () => void
  onReplay: () => void
  onViewLeaderboard: () => void
  reward: number
  rewardReceipt?: RewardReceipt | null
  huntId?: number
  playerAddress?: string
}

export function GameCompleteModal({
  isOpen,
  onClose,
  onGoHome,
  onReplay,
  onViewLeaderboard,
  reward,
  rewardReceipt,
  huntId,
  playerAddress,
}: GameCompleteModalProps) {
  const { price: xlmUsdPrice } = useXlmUsdPrice()

  const currencyFormatter = new Intl.NumberFormat(undefined, {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 2,
  })

  const usdEquivalent =
    xlmUsdPrice != null ? currencyFormatter.format(reward * xlmUsdPrice) : null

  const certificateRef = useRef<HTMLDivElement>(null)

  const {
    newAchievements,
    levelUpData,
    isLevelUpModalOpen,
    setIsLevelUpModalOpen,
    latestAttempt,
  } = useGameCompleteEffects(isOpen, playerAddress, huntId, reward)

  const completionTimeLabel = latestAttempt
    ? formatDuration(latestAttempt.totalTimeSeconds)
    : "—"

  const totalHintsUsed = latestAttempt
    ? latestAttempt.clues.reduce((sum, c) => sum + (c.hintsUsed ?? 0), 0)
    : 0

  const rankLabel = "—"

  const { data: registrationStatus } = useQuery({
    queryKey: queryKeys.registration.status(huntId, playerAddress),
    queryFn: () =>
      huntId && playerAddress ? checkRegistrationStatus(huntId, playerAddress) : null,
    enabled: isOpen && !!huntId && !!playerAddress,
    staleTime: Math.max(
      SOROBAN_READ_STALE_TIME_MS,
      queryCachePolicy.registrationStatus.staleTime
    ),
    gcTime: queryCachePolicy.registrationStatus.gcTime,
    refetchInterval: queryCachePolicy.registrationStatus.refetchInterval,
    refetchIntervalInBackground: true,
  })

  const playerProgress = registrationStatus?.progressData
    ? {
        is_completed: registrationStatus.progressData.completed,
        reward_claimed: registrationStatus.progressData.reward_claimed,
        hunt_id: huntId,
        reward_amount: reward,
      }
    : undefined

  return (
    <>
      <Dialog open={isOpen} onOpenChange={onClose}>
        <DialogContent className="sm:max-w-md text-center">
          <DialogHeader>
            <DialogTitle className="bg-gradient-to-br from-[#2F2FFF] to-[#E87785] bg-clip-text text-transparent text-2xl font-bold mb-4 text-center">
              Game Complete
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <p className="bg-gradient-to-b from-[#576065] to-[#787884] bg-clip-text text-transparent text-2xl font-normal">
              You successfully completed TDH&apos;s Crossword
            </p>

            <div className="flex items-center justify-center gap-2 text-2xl">
              <span>🥇</span>
              <span className="bg-gradient-to-b from-[#3737A4] to-[#0C0C4F] bg-clip-text text-transparent text-2xl font-bold">
                1st Place
              </span>
            </div>

            <GameCompleteStats
              completionTimeLabel={completionTimeLabel}
              totalHintsUsed={totalHintsUsed}
              rankLabel={rankLabel}
            />

            <div className="flex items-center justify-center gap-2 w-full">
              <p className="bg-gradient-to-b from-[#3737A4] to-[#0C0C4F] bg-clip-text text-transparent text-xl font-normal mb-2">
                You won
              </p>
              <div className="flex flex-col items-center gap-1">
                <div className="flex items-center justify-center gap-2 bg-[#e5e5eb] p-2 rounded-xl w-[230px]">
                  <Coin />
                  <span className="font-bold text-lg">{reward}</span>
                </div>
                {usdEquivalent && (
                  <span className="text-sm text-slate-500">≈ {usdEquivalent}</span>
                )}
              </div>
            </div>

            {rewardReceipt && (
              <GameCompleteRewardReceipt rewardReceipt={rewardReceipt} />
            )}

            <GameCompleteAchievements newAchievements={newAchievements} />

            {playerProgress && (
              <div className="mt-6 border-t border-slate-100 pt-6">
                <p className="mb-2 text-sm font-semibold text-slate-800">Claim your reward</p>
                <RewardsPanel rewards={[]} playerProgress={playerProgress} />
              </div>
            )}

            <div className="mt-6 border-t border-slate-100 pt-6">
              <NftMintProgress
                huntId={huntId ?? 0}
                rank={1}
                recipientAddress={playerAddress}
              />
            </div>

            <div className="flex gap-4">
              <div className="flex-1 p-[2px] bg-gradient-to-br from-[#4A4AFF] to-[#0C0C4F] rounded-xl">
                <Button
                  onClick={onGoHome}
                  variant="outline"
                  className="w-full h-full bg-white border-none shadow-none rounded-xl"
                  style={{ background: "white" }}
                >
                  <span className="bg-gradient-to-br from-[#4A4AFF] to-[#0C0C4F] bg-clip-text text-transparent font-bold cursor-pointer">
                    Go Home
                  </span>
                </Button>
              </div>
              <Button
                onClick={onReplay}
                className="flex-1 bg-gradient-to-br from-[#E3225C] to-[#7B1C4A] hover:bg-pink-600 text-white cursor-pointer rounded-xl"
              >
                <Replay /> Replay
              </Button>
            </div>

            <div className="flex flex-col gap-3 pt-2">
              <ShareAchievementMenu
                huntId={huntId}
                certificateRef={certificateRef}
                isRegistered={!!registrationStatus?.progressData?.hunt_id}
              />

              <ShareResultCardMenu
                huntId={huntId}
                playerAddress={playerAddress}
                latestAttempt={latestAttempt}
                rewardReceipt={rewardReceipt}
              />

              <Button
                onClick={onViewLeaderboard}
                className="w-full bg-gradient-to-b from-[#FFD43E] to-[#EC7F00] text-white text-xl font-black cursor-pointer rounded-xl h-11"
              >
                See Leaderboard
              </Button>

              <HuntReviewForm
                huntId={huntId}
                playerAddress={playerAddress}
                isOpen={isOpen}
              />
            </div>
          </div>

          <div className="fixed left-[-9999px] top-0 pointer-events-none">
            <AchievementCertificate
              ref={certificateRef}
              playerName={
                playerAddress
                  ? `${playerAddress.slice(0, 6)}...${playerAddress.slice(-4)}`
                  : "Explorer"
              }
              huntTitle={
                registrationStatus?.progressData?.hunt_id
                  ? `Hunt #${huntId}`
                  : "Scavenger Hunt"
              }
              points={reward}
              rank={1}
            />
          </div>
        </DialogContent>
      </Dialog>

      {levelUpData && (
        <LevelUpModal
          isOpen={isLevelUpModalOpen}
          onClose={() => setIsLevelUpModalOpen(false)}
          oldLevel={levelUpData.oldLevel}
          newLevel={levelUpData.newLevel}
          oldTier={levelUpData.oldTier}
          newTier={levelUpData.newTier}
        />
      )}
    </>
  )
}
