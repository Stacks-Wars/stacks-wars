"use client";

import { useEffect, useState } from "react";
import { ApiClient } from "@/lib/api/client";
import { useUser } from "@/lib/stores/user";
import LobbyCard, { LobbyCardSkeleton } from "@/components/main/lobby-card";
import type { UnclaimedReward } from "@/lib/definitions";

interface UnclaimedRewardsProps {
	userId: string;
}

export default function UnclaimedRewards({ userId }: UnclaimedRewardsProps) {
	const user = useUser();
	const [rewards, setRewards] = useState<UnclaimedReward[]>([]);
	const [loading, setLoading] = useState(false);

	useEffect(() => {
		if (user?.id === userId) {
			setLoading(true);
			ApiClient.get<UnclaimedReward[]>("/api/session/unclaimed-reward")
				.then((res) => {
					if (res.data) setRewards(res.data);
				})
				.finally(() => setLoading(false));
		}
	}, [user?.id, userId]);

	if (!user || user.id !== userId || rewards.length === 0) return null;

	return (
		<div className="mt-8 px-4 sm:mt-12 sm:px-0">
			<h2 className="mb-4 text-xl font-bold sm:mb-6 sm:text-3xl">
				Unclaimed Rewards
			</h2>
			<div className="grid grid-cols-1 gap-4 sm:gap-6">
				{loading ? (
					<LobbyCardSkeleton />
				) : (
					rewards.map((reward, index) => (
						<LobbyCard
							key={index}
							lobbyInfo={reward.lobbyInfo}
							buttonText="Claim"
							prize={reward.prize}
						/>
					))
				)}
			</div>
		</div>
	);
}
